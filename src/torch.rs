use crate::ffi;
use crate::VaccelId;
use crate::client::VsockClient;
use crate::resources::VaccelResource;
use crate::torch::{Code, DataType};
use crate::{Error, Result};

use vaccel::torch
use protobuf::ProtobufEnum;
use protocols::torch::{TorchDataType, TorchTensor};

use std::any::Any;
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};
use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq)]
// This Tensor should be same as the vaccel tensorflow Tensor
// difference: owned - bool -> uint8_t,  dims - long long int -> int64_t
pub struct Tensor <T: TensorType> {
        inner: *mut ffi::vaccel_torch_tensor,
        dims: Vec<u64>,
        data_count: usize,
        data: Vec<T>,
}

pub trait TensorType:Default + Clone {
    // DataType - should be in mod.rs?
    fn data_type() -> DataType;

    // Unit value of type
    fn one() -> self;
   
    // Zero value of type
    fn zero() -> self;
}

// What should we do with the product func?
fn product(values: &[u64]) -> u64 {
    values.iter().product()
}

// vaccel_torch_buffer, bufferLength was required
pub struct Buffer {
    inner: *mut ffi::vaccel_torch_buffer,
    vaccel_owned: bool,
}

// Struct for the pytorch model - vaccel_torch_saved_model, model path was required 
pub struct SavedModel {
    inner: *mut ffi::veccel_torch_saved_model,
}

pub struct TorchJitLoadForward {
   inner: *mut ffi::vaccel_torch_jitload_forward, 
}

// TensorType, refers to TFTensor
impl<T: TensorType> Deref for Tensor<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        if self.inner.is_null() {
            &[]
        } else {
            let data = unsafe { (*self.inner).data } as *const T;
            unsafe { std::slice::from_raw_parts(data, self.data_count) }
        }
    }
}

impl<T: TensorType> DerefMut for Tensor<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        if self.inner.is_null() {
            &mut []
        } else {
            let data = unsafe { (*self.inner).data } as *mut T;
            unsafe { std::slice::from_raw_parts_mut(data, self.data_count) }
        }
    }
}

impl<T: TensorType> Tensor<T> {
    pub fn new(dims: &[u64]) -> Self {
        let dims = Vec::from(dims);
        let data_count = product(&dims) as usize;
        let mut data = Vec::with_capacity(data_count);
        data.resize(data_count, T::zero());

        let inner = unsafe {
            ffi::vaccel_torch_tensor_new(
                dims.len() as i32,
                dims.as_ptr() as *mut _,
                T::data_type().to_int(),
            )
        };

        unsafe {
            ffi::vaccel_torch_tensor_set_data(
                inner,
                data.as_ptr() as *mut _,
                (data.len() * std::mem::size_of::<T>()) as u64,
            )
        };

        Tensor {
            inner,
            dims,
            data_count,
            data,
        }
    }

    pub unsafe fn from_vaccel_tensor(tensor: *mut ffi::vaccel_torch_tensor) -> Result<Tensor<T>> {
        if tensor.is_null() {
            return Err(Error::InvalidArgument);
        }

        if DataType::from_int((*tensor).data_type) != T::data_type() {
            return Err(Error::InvalidArgument);
        }

        let dims = std::slice::from_raw_parts((*tensor).dims as *mut _, (*tensor).nr_dims as usize);

        let data_count = product(&dims) as usize;

        let ptr = ffi::vaccel_torch_tensor_get_data(tensor);
        let data = if ptr.is_null() {
            let mut data = Vec::with_capacity(data_count);
            data.resize(data_count, T::zero());
            data
        } else {
            let data =
                std::slice::from_raw_parts(ptr as *mut T, data_count * std::mem::size_of::<T>());
            Vec::from(data)
        };

        Ok(Tensor::<T> {
            inner: tensor,
            dims: Vec::from(dims),
            data_count,
            data,
        })
    }

    pub fn with_data(mut self, data: &[T]) -> Result<Self> {
        if data.len() != self.data_count {
            return Err(Error::InvalidArgument);
        }

        for (e, v) in self.iter_mut().zip(data) {
            e.clone_from(v);
        }

        Ok(self)
    }

    pub fn nr_dims(&self) -> u64 {
        self.dims.len() as u64
    }

    pub fn dim(&self, idx: usize) -> Result<u64> {
        if idx >= self.dims.len() {
            return Err(Error::TensorFlow(Code::OutOfRange));
        }

        Ok(self.dims[idx])
    }

    pub fn data_type(&self) -> DataType {
        T::data_type()
    }

    pub fn as_grpc(&self) -> TorchTensor {
        let data = unsafe {
            std::slice::from_raw_parts((*self.inner).data as *const u8, (*self.inner).size as usize)
        };

        TorchTensor {
            data: data.to_owned(),
            dims: self.dims.clone(),
            field_type: TorchDataType::from_i32(self.data_type().to_int() as i32).unwrap(),
            ..Default::default()
        }
    }
}

impl<T: TensorType> Drop for Tensor<T> {
    fn drop(&mut self) {
        if self.inner.is_null() {
            return;
        }

        unsafe { ffi::vaccel_torch_tensor_destroy(self.inner) };
        self.inner = std::ptr::null_mut();
    }
}

pub trait TensorAny {
    fn inner(&self) -> *const ffi::vaccel_torch_tensor;

    fn inner_mut(&mut self) -> *mut ffi::vaccel_torch_tensor;

    fn data_type(&self) -> DataType;
}

impl<T: TensorType> TensorAny for Tensor<T> {
    fn inner(&self) -> *const ffi::vaccel_torch_tensor {
        self.inner
    }

    fn inner_mut(&mut self) -> *mut ffi::vaccel_torch_tensor {
        self.inner
    }

    fn data_type(&self) -> DataType {
        T::data_type()
    }
}

impl TensorAny for TorchTensor {
    fn inner(&self) -> *const ffi::vaccel_torch_tensor {
        let inner = unsafe {
            ffi::vaccel_torch_tensor_new(
                self.get_dims().len() as i32,
                self.get_dims().as_ptr() as *mut _,
                self.get_field_type().value() as u32,
            )
        };

        let size = self.get_data().len() as u64;
        let data = self.get_data().to_owned();

        unsafe { ffi::vaccel_torch_tensor_set_data(inner, data.as_ptr() as *mut libc::c_void, size) };

        std::mem::forget(data);

        inner
    }

    fn inner_mut(&mut self) -> *mut ffi::vaccel_torch_tensor {
        let inner = unsafe {
            ffi::vaccel_torch_tensor_new(
                self.get_dims().len() as i32,
                self.get_dims().as_ptr() as *mut _,
                self.get_field_type().value() as u32,
            )
        };

        let size = self.get_data().len() as u64;
        let data = self.get_data().to_owned();

        unsafe { ffi::vaccel_torch_tensor_set_data(inner, data.as_ptr() as *mut libc::c_void, size) };

        std::mem::forget(data);

        inner
    }

    fn data_type(&self) -> DataType {
        DataType::from_int(self.get_field_type().value() as u32)
    }
}

impl TensorAny for *mut ffi::vaccel_torch_tensor {
    fn inner(&self) -> *const ffi::vaccel_torch_tensor {
        *self
    }

    fn inner_mut(&mut self) -> *mut ffi::vaccel_torch_tensor {
        *self
    }

    fn data_type(&self) -> DataType {
        DataType::from_int(unsafe { (**self).data_type })
    }
}

impl TensorType for f32 {
    fn data_type() -> DataType {
        DataType::Float
    }

    fn one() -> Self {
        1.0f32
    }

    fn zero() -> Self {
        0.0f32
    }
}

impl TensorType for f64 {
    fn data_type() -> DataType {
        DataType::Double
    }

    fn one() -> Self {
        1.0f64
    }

    fn zero() -> Self {
        0.0f64
    }
}

impl TensorType for i32 {
    fn data_type() -> DataType {
        DataType::Int32
    }

    fn one() -> Self {
        1i32
    }

    fn zero() -> Self {
        0i32
    }
}

impl TensorType for u8 {
    fn data_type() -> DataType {
        DataType::UInt8
    }

    fn one() -> Self {
        1u8
    }

    fn zero() -> Self {
        0u8
    }
}

impl TensorType for i16 {
    fn data_type() -> DataType {
        DataType::Int16
    }

    fn one() -> Self {
        1i16
    }

    fn zero() -> Self {
        0i16
    }
}

impl TensorType for i8 {
    fn data_type() -> DataType {
        DataType::Int8
    }

    fn one() -> Self {
        1i8
    }

    fn zero() -> Self {
        0i8
    }
}

impl TensorType for i64 {
    fn data_type() -> DataType {
        DataType::Int64
    }

    fn one() -> Self {
        1i64
    }

    fn zero() -> Self {
        0i64
    }
}

impl TensorType for u16 {
    fn data_type() -> DataType {
        DataType::UInt16
    }

    fn one() -> Self {
        1u16
    }

    fn zero() -> Self {
        0u16
    }
}

impl TensorType for u32 {
    fn data_type() -> DataType {
        DataType::UInt32
    }

    fn one() -> Self {
        1u32
    }

    fn zero() -> Self {
        0u32
    }
}

impl TensorType for u64 {
    fn data_type() -> DataType {
        DataType::UInt64
    }

    fn one() -> Self {
        1u64
    }

    fn zero() -> Self {
        0u64
    }
}

impl TensorType for bool {
    fn data_type() -> DataType {
        DataType::Bool
    }

    fn one() -> Self {
        true
    }

    fn zero() -> Self {
        false
    }
}

impl From<&ffi::vaccel_torch_tensor> for TorchTensor {
    fn from(tensor: &ffi::vaccel_torch_tensor) -> Self {
        unsafe {
            TorchTensor {
                dims: std::slice::from_raw_parts(tensor.dims as *mut u64, tensor.nr_dims as usize)
                    .to_owned(),
                field_type: TorchDataType::from_i32((*tensor).data_type as i32).unwrap(),
                data: std::slice::from_raw_parts(tensor.data as *mut u8, tensor.size as usize)
                    .to_owned(),
                ..Default::default()
            }
        }
    }
}

/*------------------------------*/

impl Buffer {
    pub fn new(data: &[u8]) -> Self {
        let inner =
            unsafe { ffi::vaccel_torch_buffer_new(data.as_ptr() as *mut _, data.len() as u64) };
        assert!(!inner.is_null(), "Memory allocation failure");

        Buffer {
            inner,
            vaccel_owned: false,
        }
    }

    pub unsafe fn from_vaccel_buffer(buffer: *mut ffi::vaccel_torch_buffer) -> Result<Self> {
        let mut size = Default::default();
        let data = ffi::vaccel_torch_buffer_get_data(buffer, &mut size);
        if data.is_null() || size == 0 {
            return Err(Error::InvalidArgument);
        }

        Ok(Buffer {
            inner: buffer,
            vaccel_owned: true,
        })
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let mut size = Default::default();
            let ptr = ffi::vaccel_torch_buffer_get_data(self.inner, &mut size) as *const u8;
            std::slice::from_raw_parts(ptr, size as usize)
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            let mut size = Default::default();
            let ptr = ffi::vaccel_torch_buffer_get_data(self.inner, &mut size) as *mut u8;
            std::slice::from_raw_parts_mut(ptr, size as usize)
        }
    }

    pub(crate) fn inner(&self) -> *const ffi::vaccel_torch_buffer {
        self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> *mut ffi::vaccel_torch_buffer {
        self.inner
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if !self.vaccel_owned {
            // Data is not owned from vaccel runtime. Unset it from
            // the buffer so we avoid double free.
            let mut size = Default::default();
            unsafe { ffi::vaccel_torch_buffer_take_data(self.inner, &mut size) };
        }

        unsafe { ffi::vaccel_torch_buffer_destroy(self.inner) }
    }
}

impl Deref for Buffer {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

/*------------------------------*/


// Function for saved model - vaccel_torch_saved_model_new
// Create - SetPath - Destroy
impl SavedModel {
    // New Saved Model Object
    pub fn new() -> Self {
        SavedModel {
            inner: unsafe { ffi::vaccel_torch_saved_model_new() },
        }
    }

    // Create a new SavedModel from a vaccel saved model type
    pub fn id(&self) -> VaccelId {
        let inner = unsafe { ffi::vaccel_torch_saved_model_id(self.inner) };
        VaccelId::from(inner)
    }

    // Return True if already been initialized
    pub fn initialized(&self) -> bool {
        self.id().has_id()
    }

    pub fn destory(&mut self) -> Result<()> {
        if !self.initialized() {
            return Ok(());
        }

        match unsafe { ffi::vaccel_torch_saved_model_destroy(self.inner) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    fn set_path(&mut self, path: &Path) -> Result<()> {
        let c_path = CString::new(path.as_os_str().to_str().ok_or(Error::InvalidArgument)?).map_err(|_| Error::InvalidArgument)?;

        match  unsafe { ffi::vaccel_torch_saved_model_set_path(self.inner, c_path.into_raw()) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err  => Err(Error::Runtime(err)),
      }
    }

    // Create Resource from the exported saved model
    pub fn from_export_dir(mut self, path: &Path) -> Result<Self> {
        self.set_path(path)?;
        match unsafe { ffi::vaccel_torch_saved_model_register(self.inner) } as u32 {
            ffi::VACCEL_OK => Ok(self),
            err => Err(Error::Runtime(err)),
        }
    }

    // Set the in-memory protobuf data
    fn set_protobuf(&mut self, data: &[u8]) -> Result<()> {
        match unsafe {
            ffi::vaccel_torch_saved_model_set_model(self.inner, 
                                                 data.as_ptr(), 
                                                 data.len() as u64) 
                as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    // Create Resource from in-memory data
    pub fn from_in_memory(
        mut self,
        protobuf: &[u8],
    ) -> Result<Self> {
        self.set_protobuf(&protobuf)?;
        match unsafe { ffi::vaccel_torch_saved_model_register(self.inner) } as u32 {
            ffi::VACCEL_OK => Ok(self),
            err => Err(Error::Runtime(err)),
        }
    }

    pub(crate) fn inner(&self) -> *const ffi::vaccel_torch_saved_model {
        self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> *mut ffi::vaccel_torch_saved_model {
        self.inner
    }
    
    // Get the path
    pub fn get_path(&self) -> Option<PathBuf> {
        let path_str = match unsafe {
        CStr::from_ptr(ffi::vaccel_torch_saved_model_get_path(self.inner)).to_str()
    }   {
            Ok(s) => s,
            Err(_) => return None,
        };

        Some(PathBuf::from(path_str))
    }

    // Get the data of the protobuf
    pub fn get_protobuf(&self) -> Option<&[u8]> {
        let mut size = Default::default();
        let ptr = unsafe { ffi::vaccel_torch_saved_model_get_model(self.inner, &mut size) };
        if !ptr.is_null() {
            Some(unsafe { std::slice::from_raw_parts(ptr, size as usize) })
        } else 
                {
                    None
                }
    }
}

    impl crate::resource::Resource for SavedModel {
        fn id(&self) -> VaccelId {
            self.id()
        }

        fn initialized(&self) -> bool {
            self.initialized()
        }

        fn to_vaccel_ptr(&self) -> Option<*const ffi::vaccel_resource> {
            if !self.initialized() {
                None
            } else {
                let resource = unsafe { (*self.inner).resource };
                Some(resource)
            }
        }

        fn to_mut_vaccel_ptr(&self) -> Option<*mut ffi::vaccel_resource> {
            if !self.initialized() {
                None
            } else {
                let resource = unsafe { (*self.inner).resource };
                Some(resource)
            }
        }

        fn destroy(&mut self) -> Result<()> {
            self.destroy()
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_mut_any(&mut self) -> &mut dyn Any {
            self
        }
    }   

/*------------------------------*/

// Function for the torch jitload
impl TorchJitLoadForward {
    
}
