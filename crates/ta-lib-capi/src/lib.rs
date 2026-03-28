#![allow(non_snake_case)]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ffi::c_char;
use core::slice;

use ta_lib::{
    CandleSettingType, Compatibility, FuncUnstId, RangeType, RetCode, UPSTREAM_TA_LIB_DESCRIBE,
};

#[repr(C)]
pub struct TA_StringTable {
    pub size: u32,
    pub string: *const *const c_char,
    pub hiddenData: *mut core::ffi::c_void,
}
unsafe impl Sync for TA_StringTable {}

#[repr(C)]
pub struct TA_FuncInfo {
    pub name: *const c_char,
    pub group: *const c_char,
    pub hint: *const c_char,
    pub camelCaseName: *const c_char,
    pub flags: i32,
    pub nbInput: u32,
    pub nbOptInput: u32,
    pub nbOutput: u32,
    pub handle: *const u32,
}
unsafe impl Sync for TA_FuncInfo {}

#[repr(C)]
pub struct TA_InputParameterInfo {
    pub type_: i32,
    pub paramName: *const c_char,
    pub flags: i32,
}
unsafe impl Sync for TA_InputParameterInfo {}

#[repr(C)]
pub struct TA_OptInputParameterInfo {
    pub type_: i32,
    pub paramName: *const c_char,
    pub flags: i32,
    pub displayName: *const c_char,
    pub dataSet: *const core::ffi::c_void,
    pub defaultValue: f64,
    pub hint: *const c_char,
    pub helpFile: *const c_char,
}
unsafe impl Sync for TA_OptInputParameterInfo {}

#[repr(C)]
pub struct TA_OutputParameterInfo {
    pub type_: i32,
    pub paramName: *const c_char,
    pub flags: i32,
}
unsafe impl Sync for TA_OutputParameterInfo {}

#[repr(C)]
pub struct TA_ParamHolder {
    pub hiddenData: *mut core::ffi::c_void,
}

type TaCallForEachFunc = Option<unsafe extern "C" fn(*const TA_FuncInfo, *mut core::ffi::c_void)>;

const TA_INPUT_PRICE: i32 = 0;
const TA_INPUT_REAL: i32 = 1;
const TA_INPUT_INTEGER: i32 = 2;

const TA_OPTINPUT_REAL_RANGE: i32 = 0;
const TA_OPTINPUT_REAL_LIST: i32 = 1;
const TA_OPTINPUT_INTEGER_RANGE: i32 = 2;
const TA_OPTINPUT_INTEGER_LIST: i32 = 3;

const TA_OUTPUT_REAL: i32 = 0;
const TA_OUTPUT_INTEGER: i32 = 1;

#[repr(C)]
pub struct TA_RetCodeInfo {
    pub enumStr: *const c_char,
    pub infoStr: *const c_char,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct PriceInputPtrs {
    open: *const f64,
    high: *const f64,
    low: *const f64,
    close: *const f64,
    volume: *const f64,
    open_interest: *const f64,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
enum InputSlot {
    Unset,
    Real(*const f64),
    Integer(*const i32),
    Price(PriceInputPtrs),
}

#[derive(Clone, Copy)]
enum OptValue {
    Unset,
    Integer(i32),
    Real(f64),
}

#[derive(Clone, Copy)]
enum OutputSlot {
    Unset,
    Real(*mut f64),
    Integer(*mut i32),
}

struct ParamHolderData {
    function_index: usize,
    inputs: Vec<InputSlot>,
    opt_inputs: Vec<OptValue>,
    outputs: Vec<OutputSlot>,
}

impl ParamHolderData {
    fn from_function_info(function_info: &TA_FuncInfo) -> Self {
        Self {
            function_index: abstract_function_index_from_handle(function_info.handle)
                .expect("generated function handle"),
            inputs: vec![InputSlot::Unset; function_info.nbInput as usize],
            opt_inputs: vec![OptValue::Unset; function_info.nbOptInput as usize],
            outputs: vec![OutputSlot::Unset; function_info.nbOutput as usize],
        }
    }

    fn real_input(&self, index: usize) -> Result<*const f64, RetCode> {
        match self.inputs.get(index).copied() {
            Some(InputSlot::Real(ptr)) if !ptr.is_null() => Ok(ptr),
            Some(InputSlot::Unset) => Err(RetCode::InputNotAllInitialize),
            Some(_) => Err(RetCode::InvalidParamHolderType),
            None => Err(RetCode::BadParam),
        }
    }

    #[allow(dead_code)]
    fn integer_input(&self, index: usize) -> Result<*const i32, RetCode> {
        match self.inputs.get(index).copied() {
            Some(InputSlot::Integer(ptr)) if !ptr.is_null() => Ok(ptr),
            Some(InputSlot::Unset) => Err(RetCode::InputNotAllInitialize),
            Some(_) => Err(RetCode::InvalidParamHolderType),
            None => Err(RetCode::BadParam),
        }
    }

    fn price_input(&self, index: usize) -> Result<PriceInputPtrs, RetCode> {
        match self.inputs.get(index).copied() {
            Some(InputSlot::Price(price)) => Ok(price),
            Some(InputSlot::Unset) => Err(RetCode::InputNotAllInitialize),
            Some(_) => Err(RetCode::InvalidParamHolderType),
            None => Err(RetCode::BadParam),
        }
    }

    fn opt_int(&self, index: usize, default_value: i32) -> Result<i32, RetCode> {
        match self.opt_inputs.get(index).copied() {
            Some(OptValue::Unset) => Ok(default_value),
            Some(OptValue::Integer(value)) => Ok(value),
            Some(_) => Err(RetCode::InvalidParamHolderType),
            None => Err(RetCode::BadParam),
        }
    }

    fn opt_real(&self, index: usize, default_value: f64) -> Result<f64, RetCode> {
        match self.opt_inputs.get(index).copied() {
            Some(OptValue::Unset) => Ok(default_value),
            Some(OptValue::Real(value)) => Ok(value),
            Some(_) => Err(RetCode::InvalidParamHolderType),
            None => Err(RetCode::BadParam),
        }
    }

    fn real_output(&self, index: usize) -> Result<*mut f64, RetCode> {
        match self.outputs.get(index).copied() {
            Some(OutputSlot::Real(ptr)) if !ptr.is_null() => Ok(ptr),
            Some(OutputSlot::Unset) => Err(RetCode::OutputNotAllInitialize),
            Some(_) => Err(RetCode::InvalidParamHolderType),
            None => Err(RetCode::BadParam),
        }
    }

    fn integer_output(&self, index: usize) -> Result<*mut i32, RetCode> {
        match self.outputs.get(index).copied() {
            Some(OutputSlot::Integer(ptr)) if !ptr.is_null() => Ok(ptr),
            Some(OutputSlot::Unset) => Err(RetCode::OutputNotAllInitialize),
            Some(_) => Err(RetCode::InvalidParamHolderType),
            None => Err(RetCode::BadParam),
        }
    }
}

struct OwnedStringTable {
    values: Box<[*const c_char]>,
}

const VERSION_STRING: &[u8] = b"ta-lib-rust\0";
const RC_SUCCESS_ENUM: &[u8] = b"TA_SUCCESS\0";
const RC_SUCCESS_INFO: &[u8] = b"No error\0";
const RC_LIB_NOT_INITIALIZE_ENUM: &[u8] = b"TA_LIB_NOT_INITIALIZE\0";
const RC_LIB_NOT_INITIALIZE_INFO: &[u8] = b"TA_Initialize was not successfully called\0";
const RC_BAD_PARAM_ENUM: &[u8] = b"TA_BAD_PARAM\0";
const RC_BAD_PARAM_INFO: &[u8] = b"A parameter is out of range\0";
const RC_ALLOC_ERR_ENUM: &[u8] = b"TA_ALLOC_ERR\0";
const RC_ALLOC_ERR_INFO: &[u8] = b"Allocation failed\0";
const RC_GROUP_NOT_FOUND_ENUM: &[u8] = b"TA_GROUP_NOT_FOUND\0";
const RC_GROUP_NOT_FOUND_INFO: &[u8] = b"Group not found\0";
const RC_FUNC_NOT_FOUND_ENUM: &[u8] = b"TA_FUNC_NOT_FOUND\0";
const RC_FUNC_NOT_FOUND_INFO: &[u8] = b"Function not found\0";
const RC_INVALID_HANDLE_ENUM: &[u8] = b"TA_INVALID_HANDLE\0";
const RC_INVALID_HANDLE_INFO: &[u8] = b"Invalid handle\0";
const RC_INVALID_PARAM_HOLDER_ENUM: &[u8] = b"TA_INVALID_PARAM_HOLDER\0";
const RC_INVALID_PARAM_HOLDER_INFO: &[u8] = b"Invalid parameter holder\0";
const RC_INVALID_PARAM_HOLDER_TYPE_ENUM: &[u8] = b"TA_INVALID_PARAM_HOLDER_TYPE\0";
const RC_INVALID_PARAM_HOLDER_TYPE_INFO: &[u8] = b"Invalid parameter holder type\0";
const RC_INVALID_PARAM_FUNCTION_ENUM: &[u8] = b"TA_INVALID_PARAM_FUNCTION\0";
const RC_INVALID_PARAM_FUNCTION_INFO: &[u8] = b"Invalid parameter function\0";
const RC_INPUT_NOT_ALL_INITIALIZE_ENUM: &[u8] = b"TA_INPUT_NOT_ALL_INITIALIZE\0";
const RC_INPUT_NOT_ALL_INITIALIZE_INFO: &[u8] = b"Input parameters not all initialized\0";
const RC_OUTPUT_NOT_ALL_INITIALIZE_ENUM: &[u8] = b"TA_OUTPUT_NOT_ALL_INITIALIZE\0";
const RC_OUTPUT_NOT_ALL_INITIALIZE_INFO: &[u8] = b"Output parameters not all initialized\0";
const RC_OUT_OF_RANGE_START_INDEX_ENUM: &[u8] = b"TA_OUT_OF_RANGE_START_INDEX\0";
const RC_OUT_OF_RANGE_START_INDEX_INFO: &[u8] = b"Start index out of range\0";
const RC_OUT_OF_RANGE_END_INDEX_ENUM: &[u8] = b"TA_OUT_OF_RANGE_END_INDEX\0";
const RC_OUT_OF_RANGE_END_INDEX_INFO: &[u8] = b"End index out of range\0";
const RC_INVALID_LIST_TYPE_ENUM: &[u8] = b"TA_INVALID_LIST_TYPE\0";
const RC_INVALID_LIST_TYPE_INFO: &[u8] = b"Invalid list type\0";
const RC_BAD_OBJECT_ENUM: &[u8] = b"TA_BAD_OBJECT\0";
const RC_BAD_OBJECT_INFO: &[u8] = b"Bad object\0";
const RC_NOT_SUPPORTED_ENUM: &[u8] = b"TA_NOT_SUPPORTED\0";
const RC_NOT_SUPPORTED_INFO: &[u8] = b"Operation not supported\0";
const RC_INTERNAL_ERROR_ENUM: &[u8] = b"TA_INTERNAL_ERROR\0";
const RC_INTERNAL_ERROR_INFO: &[u8] = b"Internal error\0";
const RC_UNKNOWN_ERR_ENUM: &[u8] = b"TA_UNKNOWN_ERR\0";
const RC_UNKNOWN_ERR_INFO: &[u8] = b"Unknown error\0";

fn c_str(bytes: &'static [u8]) -> *const c_char {
    bytes.as_ptr().cast()
}

fn retcode_info(ret_code: RetCode) -> (*const c_char, *const c_char) {
    match ret_code {
        RetCode::Success => (c_str(RC_SUCCESS_ENUM), c_str(RC_SUCCESS_INFO)),
        RetCode::LibNotInitialize => (
            c_str(RC_LIB_NOT_INITIALIZE_ENUM),
            c_str(RC_LIB_NOT_INITIALIZE_INFO),
        ),
        RetCode::BadParam => (c_str(RC_BAD_PARAM_ENUM), c_str(RC_BAD_PARAM_INFO)),
        RetCode::AllocErr => (c_str(RC_ALLOC_ERR_ENUM), c_str(RC_ALLOC_ERR_INFO)),
        RetCode::GroupNotFound => (
            c_str(RC_GROUP_NOT_FOUND_ENUM),
            c_str(RC_GROUP_NOT_FOUND_INFO),
        ),
        RetCode::FuncNotFound => (c_str(RC_FUNC_NOT_FOUND_ENUM), c_str(RC_FUNC_NOT_FOUND_INFO)),
        RetCode::InvalidHandle => (c_str(RC_INVALID_HANDLE_ENUM), c_str(RC_INVALID_HANDLE_INFO)),
        RetCode::InvalidParamHolder => (
            c_str(RC_INVALID_PARAM_HOLDER_ENUM),
            c_str(RC_INVALID_PARAM_HOLDER_INFO),
        ),
        RetCode::InvalidParamHolderType => (
            c_str(RC_INVALID_PARAM_HOLDER_TYPE_ENUM),
            c_str(RC_INVALID_PARAM_HOLDER_TYPE_INFO),
        ),
        RetCode::InvalidParamFunction => (
            c_str(RC_INVALID_PARAM_FUNCTION_ENUM),
            c_str(RC_INVALID_PARAM_FUNCTION_INFO),
        ),
        RetCode::InputNotAllInitialize => (
            c_str(RC_INPUT_NOT_ALL_INITIALIZE_ENUM),
            c_str(RC_INPUT_NOT_ALL_INITIALIZE_INFO),
        ),
        RetCode::OutputNotAllInitialize => (
            c_str(RC_OUTPUT_NOT_ALL_INITIALIZE_ENUM),
            c_str(RC_OUTPUT_NOT_ALL_INITIALIZE_INFO),
        ),
        RetCode::OutOfRangeStartIndex => (
            c_str(RC_OUT_OF_RANGE_START_INDEX_ENUM),
            c_str(RC_OUT_OF_RANGE_START_INDEX_INFO),
        ),
        RetCode::OutOfRangeEndIndex => (
            c_str(RC_OUT_OF_RANGE_END_INDEX_ENUM),
            c_str(RC_OUT_OF_RANGE_END_INDEX_INFO),
        ),
        RetCode::InvalidListType => (
            c_str(RC_INVALID_LIST_TYPE_ENUM),
            c_str(RC_INVALID_LIST_TYPE_INFO),
        ),
        RetCode::BadObject => (c_str(RC_BAD_OBJECT_ENUM), c_str(RC_BAD_OBJECT_INFO)),
        RetCode::NotSupported => (c_str(RC_NOT_SUPPORTED_ENUM), c_str(RC_NOT_SUPPORTED_INFO)),
        RetCode::InternalError => (c_str(RC_INTERNAL_ERROR_ENUM), c_str(RC_INTERNAL_ERROR_INFO)),
        RetCode::UnknownErr => (c_str(RC_UNKNOWN_ERR_ENUM), c_str(RC_UNKNOWN_ERR_INFO)),
    }
}

fn retcode_from_i32(value: i32) -> RetCode {
    match value {
        0 => RetCode::Success,
        1 => RetCode::LibNotInitialize,
        2 => RetCode::BadParam,
        3 => RetCode::AllocErr,
        4 => RetCode::GroupNotFound,
        5 => RetCode::FuncNotFound,
        6 => RetCode::InvalidHandle,
        7 => RetCode::InvalidParamHolder,
        8 => RetCode::InvalidParamHolderType,
        9 => RetCode::InvalidParamFunction,
        10 => RetCode::InputNotAllInitialize,
        11 => RetCode::OutputNotAllInitialize,
        12 => RetCode::OutOfRangeStartIndex,
        13 => RetCode::OutOfRangeEndIndex,
        14 => RetCode::InvalidListType,
        15 => RetCode::BadObject,
        16 => RetCode::NotSupported,
        5000 => RetCode::InternalError,
        _ => RetCode::UnknownErr,
    }
}

fn normalize_range(start_idx: i32, end_idx: i32) -> Result<(usize, usize), RetCode> {
    if start_idx < 0 {
        return Err(RetCode::OutOfRangeStartIndex);
    }
    if end_idx < 0 {
        return Err(RetCode::OutOfRangeEndIndex);
    }
    let start_idx = start_idx as usize;
    let end_idx = end_idx as usize;
    if end_idx < start_idx {
        return Err(RetCode::OutOfRangeEndIndex);
    }
    Ok((start_idx, end_idx))
}

unsafe fn input_real_slice<'a>(ptr: *const f64, len: usize) -> Result<&'a [f64], RetCode> {
    if ptr.is_null() {
        return Err(RetCode::BadParam);
    }
    // SAFETY: caller provides TA-Lib input arrays with at least end_idx + 1 elements.
    Ok(unsafe { slice::from_raw_parts(ptr, len) })
}

fn output_capacity(start_idx: usize, end_idx: usize) -> usize {
    end_idx.saturating_sub(start_idx) + 1
}

fn write_output_meta(
    out_beg_idx: *mut i32,
    out_nb_element: *mut i32,
    beg_idx: usize,
    nb_element: usize,
) {
    // SAFETY: pointers are validated by generated wrappers before calling.
    unsafe {
        *out_beg_idx = beg_idx as i32;
        *out_nb_element = nb_element as i32;
    }
}

fn copy_real_output(dst: *mut f64, src: &[f64]) -> Result<(), RetCode> {
    if dst.is_null() {
        return Err(RetCode::BadParam);
    }
    // SAFETY: TA-Lib callers provide output buffers large enough for the requested range.
    unsafe { core::ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len()) };
    Ok(())
}

fn copy_integer_output(dst: *mut i32, src: &[i32]) -> Result<(), RetCode> {
    if dst.is_null() {
        return Err(RetCode::BadParam);
    }
    // SAFETY: TA-Lib callers provide output buffers large enough for the requested range.
    unsafe { core::ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len()) };
    Ok(())
}

unsafe fn c_str_arg<'a>(ptr: *const c_char) -> Result<&'a str, RetCode> {
    if ptr.is_null() {
        return Err(RetCode::BadParam);
    }
    // SAFETY: caller provides a valid null-terminated string.
    let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
    c_str.to_str().map_err(|_| RetCode::BadParam)
}

unsafe fn holder_data<'a>(params: *const TA_ParamHolder) -> Result<&'a ParamHolderData, RetCode> {
    if params.is_null() {
        return Err(RetCode::InvalidParamHolder);
    }
    // SAFETY: params was allocated by TA_ParamHolderAlloc.
    let hidden = unsafe { (*params).hiddenData.cast::<ParamHolderData>() };
    if hidden.is_null() {
        return Err(RetCode::InvalidParamHolder);
    }
    // SAFETY: hiddenData points to a live ParamHolderData allocation.
    Ok(unsafe { &*hidden })
}

unsafe fn holder_data_mut<'a>(
    params: *mut TA_ParamHolder,
) -> Result<&'a mut ParamHolderData, RetCode> {
    if params.is_null() {
        return Err(RetCode::InvalidParamHolder);
    }
    // SAFETY: params was allocated by TA_ParamHolderAlloc.
    let hidden = unsafe { (*params).hiddenData.cast::<ParamHolderData>() };
    if hidden.is_null() {
        return Err(RetCode::InvalidParamHolder);
    }
    // SAFETY: hiddenData points to a live ParamHolderData allocation.
    Ok(unsafe { &mut *hidden })
}

fn alloc_string_table(names: Vec<*const c_char>) -> *mut TA_StringTable {
    let values: Box<[*const c_char]> = names.into_boxed_slice();
    let string_ptr = values.as_ptr();
    let hidden = Box::new(OwnedStringTable { values });
    Box::into_raw(Box::new(TA_StringTable {
        size: hidden.values.len() as u32,
        string: string_ptr,
        hiddenData: Box::into_raw(hidden).cast(),
    }))
}

fn group_function_names(group: &str) -> Vec<*const c_char> {
    ABSTRACT_FUNCTION_INFOS
        .iter()
        .filter(|info| {
            if info.group.is_null() {
                return false;
            }
            // SAFETY: generated pointers point to static nul-terminated strings.
            let value = unsafe { std::ffi::CStr::from_ptr(info.group) };
            value.to_bytes() == group.as_bytes()
        })
        .map(|info| info.name)
        .collect()
}

#[unsafe(no_mangle)]
pub extern "C" fn TA_Initialize() -> i32 {
    ta_lib::initialize() as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn TA_Shutdown() -> i32 {
    ta_lib::shutdown() as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn TA_GetVersionString() -> *const c_char {
    let _ = UPSTREAM_TA_LIB_DESCRIBE;
    c_str(VERSION_STRING)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_SetRetCodeInfo(theRetCode: i32, retCodeInfo: *mut TA_RetCodeInfo) {
    if retCodeInfo.is_null() {
        return;
    }
    let (enum_str, info_str) = retcode_info(retcode_from_i32(theRetCode));
    // SAFETY: caller provided a valid TA_RetCodeInfo pointer.
    unsafe {
        (*retCodeInfo).enumStr = enum_str;
        (*retCodeInfo).infoStr = info_str;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn TA_SetCompatibility(value: i32) -> i32 {
    let Some(value) = Compatibility::from_i32(value) else {
        return RetCode::BadParam as i32;
    };
    ta_lib::Core::set_compatibility(value) as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn TA_SetUnstablePeriod(id: i32, unstable_period: u32) -> i32 {
    let Some(id) = FuncUnstId::from_i32(id) else {
        return RetCode::BadParam as i32;
    };
    ta_lib::Core::set_unstable_period(id, unstable_period) as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn TA_GetUnstablePeriod(id: i32) -> u32 {
    FuncUnstId::from_i32(id)
        .map(ta_lib::Core::get_unstable_period)
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn TA_SetCandleSettings(
    setting_type: i32,
    range_type: i32,
    avg_period: i32,
    factor: f64,
) -> i32 {
    let Some(setting_type) = CandleSettingType::from_i32(setting_type) else {
        return RetCode::BadParam as i32;
    };
    let Some(range_type) = RangeType::from_i32(range_type) else {
        return RetCode::BadParam as i32;
    };
    ta_lib::Core::set_candle_settings(setting_type, range_type, avg_period, factor) as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn TA_RestoreCandleDefaultSettings(setting_type: i32) -> i32 {
    let Some(setting_type) = CandleSettingType::from_i32(setting_type) else {
        return RetCode::BadParam as i32;
    };
    ta_lib::Core::restore_candle_default_settings(setting_type) as i32
}

include!(concat!(env!("OUT_DIR"), "/generated_c_api_wrappers.rs"));
include!(concat!(env!("OUT_DIR"), "/generated_abstract_api.rs"));

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_GroupTableAlloc(table: *mut *mut TA_StringTable) -> i32 {
    if table.is_null() {
        return RetCode::BadParam as i32;
    }
    // SAFETY: caller provides a valid output pointer.
    unsafe {
        *table = alloc_string_table(
            ABSTRACT_GROUP_NAMES
                .iter()
                .map(|name| name.as_ptr().cast())
                .collect(),
        )
    };
    RetCode::Success as i32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_GroupTableFree(table: *mut TA_StringTable) -> i32 {
    if table.is_null() {
        return RetCode::BadParam as i32;
    }
    // SAFETY: table was allocated by alloc_string_table.
    let table = unsafe { Box::from_raw(table) };
    if !table.hiddenData.is_null() {
        // SAFETY: hiddenData was created from Box<OwnedStringTable>.
        let _ = unsafe { Box::from_raw(table.hiddenData.cast::<OwnedStringTable>()) };
    }
    RetCode::Success as i32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_FuncTableAlloc(
    group: *const c_char,
    table: *mut *mut TA_StringTable,
) -> i32 {
    if table.is_null() {
        return RetCode::BadParam as i32;
    }
    let Ok(group) = (unsafe { c_str_arg(group) }) else {
        return RetCode::BadParam as i32;
    };
    if abstract_group_name_bytes(group).is_none() {
        return RetCode::GroupNotFound as i32;
    }
    // SAFETY: caller provides a valid output pointer.
    unsafe { *table = alloc_string_table(group_function_names(group)) };
    RetCode::Success as i32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_FuncTableFree(table: *mut TA_StringTable) -> i32 {
    unsafe { TA_GroupTableFree(table) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_GetFuncHandle(name: *const c_char, handle: *mut *const u32) -> i32 {
    if handle.is_null() {
        return RetCode::BadParam as i32;
    }
    let Ok(name) = (unsafe { c_str_arg(name) }) else {
        return RetCode::BadParam as i32;
    };
    let Some(index) = abstract_function_index_by_name(name) else {
        return RetCode::FuncNotFound as i32;
    };
    // SAFETY: generated function info table is static and caller provided a valid output ptr.
    unsafe { *handle = ABSTRACT_FUNCTION_INFOS[index].handle };
    RetCode::Success as i32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_GetFuncInfo(
    handle: *const u32,
    func_info: *mut *const TA_FuncInfo,
) -> i32 {
    if func_info.is_null() {
        return RetCode::BadParam as i32;
    }
    let Some(index) = abstract_function_index_from_handle(handle) else {
        return RetCode::InvalidHandle as i32;
    };
    // SAFETY: static table and valid output ptr.
    unsafe { *func_info = &ABSTRACT_FUNCTION_INFOS[index] };
    RetCode::Success as i32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_ForEachFunc(
    function_to_call: TaCallForEachFunc,
    opaque_data: *mut core::ffi::c_void,
) -> i32 {
    let Some(function_to_call) = function_to_call else {
        return RetCode::BadParam as i32;
    };
    for function_info in ABSTRACT_FUNCTION_INFOS {
        // SAFETY: callback and pointers are provided by the caller / static table.
        unsafe { function_to_call(function_info, opaque_data) };
    }
    RetCode::Success as i32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_GetInputParameterInfo(
    handle: *const u32,
    param_index: u32,
    info: *mut *const TA_InputParameterInfo,
) -> i32 {
    if info.is_null() {
        return RetCode::BadParam as i32;
    }
    let Some(function_index) = abstract_function_index_from_handle(handle) else {
        return RetCode::InvalidHandle as i32;
    };
    let Some(value) = abstract_input_info(function_index, param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    // SAFETY: valid output pointer.
    unsafe { *info = value };
    RetCode::Success as i32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_GetOptInputParameterInfo(
    handle: *const u32,
    param_index: u32,
    info: *mut *const TA_OptInputParameterInfo,
) -> i32 {
    if info.is_null() {
        return RetCode::BadParam as i32;
    }
    let Some(function_index) = abstract_function_index_from_handle(handle) else {
        return RetCode::InvalidHandle as i32;
    };
    let Some(value) = abstract_opt_input_info(function_index, param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    // SAFETY: valid output pointer.
    unsafe { *info = value };
    RetCode::Success as i32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_GetOutputParameterInfo(
    handle: *const u32,
    param_index: u32,
    info: *mut *const TA_OutputParameterInfo,
) -> i32 {
    if info.is_null() {
        return RetCode::BadParam as i32;
    }
    let Some(function_index) = abstract_function_index_from_handle(handle) else {
        return RetCode::InvalidHandle as i32;
    };
    let Some(value) = abstract_output_info(function_index, param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    // SAFETY: valid output pointer.
    unsafe { *info = value };
    RetCode::Success as i32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_ParamHolderAlloc(
    handle: *const u32,
    allocated_params: *mut *mut TA_ParamHolder,
) -> i32 {
    if allocated_params.is_null() {
        return RetCode::BadParam as i32;
    }
    let Some(function_index) = abstract_function_index_from_handle(handle) else {
        return RetCode::InvalidHandle as i32;
    };
    let function_info = &ABSTRACT_FUNCTION_INFOS[function_index];
    let data = Box::new(ParamHolderData::from_function_info(function_info));
    let holder = Box::new(TA_ParamHolder {
        hiddenData: Box::into_raw(data).cast(),
    });
    // SAFETY: valid output pointer.
    unsafe { *allocated_params = Box::into_raw(holder) };
    RetCode::Success as i32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_ParamHolderFree(params: *mut TA_ParamHolder) -> i32 {
    if params.is_null() {
        return RetCode::BadParam as i32;
    }
    // SAFETY: params was allocated by TA_ParamHolderAlloc.
    let holder = unsafe { Box::from_raw(params) };
    if !holder.hiddenData.is_null() {
        // SAFETY: hiddenData was created from Box<ParamHolderData>.
        let _ = unsafe { Box::from_raw(holder.hiddenData.cast::<ParamHolderData>()) };
    }
    RetCode::Success as i32
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_SetInputParamIntegerPtr(
    params: *mut TA_ParamHolder,
    param_index: u32,
    value: *const i32,
) -> i32 {
    let Ok(holder) = (unsafe { holder_data_mut(params) }) else {
        return RetCode::InvalidParamHolder as i32;
    };
    let Some(slot) = holder.inputs.get_mut(param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    let Some(info) = abstract_input_info(holder.function_index, param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    if info.type_ != TA_INPUT_INTEGER {
        return RetCode::BadParam as i32;
    }
    match *slot {
        InputSlot::Unset | InputSlot::Integer(_) => {
            *slot = InputSlot::Integer(value);
            RetCode::Success as i32
        }
        _ => RetCode::InvalidParamHolderType as i32,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_SetInputParamRealPtr(
    params: *mut TA_ParamHolder,
    param_index: u32,
    value: *const f64,
) -> i32 {
    let Ok(holder) = (unsafe { holder_data_mut(params) }) else {
        return RetCode::InvalidParamHolder as i32;
    };
    let Some(slot) = holder.inputs.get_mut(param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    let Some(info) = abstract_input_info(holder.function_index, param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    if info.type_ != TA_INPUT_REAL {
        return RetCode::BadParam as i32;
    }
    match *slot {
        InputSlot::Unset | InputSlot::Real(_) => {
            *slot = InputSlot::Real(value);
            RetCode::Success as i32
        }
        _ => RetCode::InvalidParamHolderType as i32,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_SetInputParamPricePtr(
    params: *mut TA_ParamHolder,
    param_index: u32,
    open: *const f64,
    high: *const f64,
    low: *const f64,
    close: *const f64,
    volume: *const f64,
    open_interest: *const f64,
) -> i32 {
    let Ok(holder) = (unsafe { holder_data_mut(params) }) else {
        return RetCode::InvalidParamHolder as i32;
    };
    let Some(slot) = holder.inputs.get_mut(param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    let Some(info) = abstract_input_info(holder.function_index, param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    if info.type_ != TA_INPUT_PRICE {
        return RetCode::BadParam as i32;
    }
    match *slot {
        InputSlot::Unset | InputSlot::Price(_) => {
            *slot = InputSlot::Price(PriceInputPtrs {
                open,
                high,
                low,
                close,
                volume,
                open_interest,
            });
            RetCode::Success as i32
        }
        _ => RetCode::InvalidParamHolderType as i32,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_SetOptInputParamInteger(
    params: *mut TA_ParamHolder,
    param_index: u32,
    value: i32,
) -> i32 {
    let Ok(holder) = (unsafe { holder_data_mut(params) }) else {
        return RetCode::InvalidParamHolder as i32;
    };
    let Some(slot) = holder.opt_inputs.get_mut(param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    let Some(info) = abstract_opt_input_info(holder.function_index, param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    if info.type_ != TA_OPTINPUT_INTEGER_RANGE && info.type_ != TA_OPTINPUT_INTEGER_LIST {
        return RetCode::BadParam as i32;
    }
    match *slot {
        OptValue::Unset | OptValue::Integer(_) => {
            *slot = OptValue::Integer(value);
            RetCode::Success as i32
        }
        _ => RetCode::InvalidParamHolderType as i32,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_SetOptInputParamReal(
    params: *mut TA_ParamHolder,
    param_index: u32,
    value: f64,
) -> i32 {
    let Ok(holder) = (unsafe { holder_data_mut(params) }) else {
        return RetCode::InvalidParamHolder as i32;
    };
    let Some(slot) = holder.opt_inputs.get_mut(param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    let Some(info) = abstract_opt_input_info(holder.function_index, param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    if info.type_ != TA_OPTINPUT_REAL_RANGE && info.type_ != TA_OPTINPUT_REAL_LIST {
        return RetCode::BadParam as i32;
    }
    match *slot {
        OptValue::Unset | OptValue::Real(_) => {
            *slot = OptValue::Real(value);
            RetCode::Success as i32
        }
        _ => RetCode::InvalidParamHolderType as i32,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_SetOutputParamIntegerPtr(
    params: *mut TA_ParamHolder,
    param_index: u32,
    out: *mut i32,
) -> i32 {
    let Ok(holder) = (unsafe { holder_data_mut(params) }) else {
        return RetCode::InvalidParamHolder as i32;
    };
    let Some(slot) = holder.outputs.get_mut(param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    let Some(info) = abstract_output_info(holder.function_index, param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    if info.type_ != TA_OUTPUT_INTEGER {
        return RetCode::BadParam as i32;
    }
    match *slot {
        OutputSlot::Unset | OutputSlot::Integer(_) => {
            *slot = OutputSlot::Integer(out);
            RetCode::Success as i32
        }
        _ => RetCode::InvalidParamHolderType as i32,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_SetOutputParamRealPtr(
    params: *mut TA_ParamHolder,
    param_index: u32,
    out: *mut f64,
) -> i32 {
    let Ok(holder) = (unsafe { holder_data_mut(params) }) else {
        return RetCode::InvalidParamHolder as i32;
    };
    let Some(slot) = holder.outputs.get_mut(param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    let Some(info) = abstract_output_info(holder.function_index, param_index as usize) else {
        return RetCode::BadParam as i32;
    };
    if info.type_ != TA_OUTPUT_REAL {
        return RetCode::BadParam as i32;
    }
    match *slot {
        OutputSlot::Unset | OutputSlot::Real(_) => {
            *slot = OutputSlot::Real(out);
            RetCode::Success as i32
        }
        _ => RetCode::InvalidParamHolderType as i32,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_GetLookback(params: *const TA_ParamHolder, lookback: *mut i32) -> i32 {
    let Ok(holder) = (unsafe { holder_data(params) }) else {
        return RetCode::InvalidParamHolder as i32;
    };
    unsafe { generated_abstract_lookback(holder, lookback) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn TA_CallFunc(
    params: *const TA_ParamHolder,
    start_idx: i32,
    end_idx: i32,
    out_beg_idx: *mut i32,
    out_nb_element: *mut i32,
) -> i32 {
    let Ok(holder) = (unsafe { holder_data(params) }) else {
        return RetCode::InvalidParamHolder as i32;
    };
    unsafe { generated_abstract_call(holder, start_idx, end_idx, out_beg_idx, out_nb_element) }
}

#[unsafe(no_mangle)]
pub extern "C" fn TA_FunctionDescriptionXML() -> *const c_char {
    static XML: &str = concat!(
        include_str!("../../../upstream-ta-lib-c/ta_func_api.xml"),
        "\0"
    );
    XML.as_ptr().cast()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_seed_function_round_trip_works() {
        assert_eq!(TA_Initialize(), RetCode::Success as i32);

        let lhs = [1.0, 2.0, 3.0];
        let rhs = [4.0, 5.0, 6.0];
        let mut out_beg = -1;
        let mut out_nb = -1;
        let mut out = [0.0; 3];

        // SAFETY: test arrays are valid for the requested range and outputs are writable.
        let ret_code = unsafe {
            TA_ADD(
                0,
                2,
                lhs.as_ptr(),
                rhs.as_ptr(),
                &mut out_beg,
                &mut out_nb,
                out.as_mut_ptr(),
            )
        };

        assert_eq!(ret_code, RetCode::Success as i32);
        assert_eq!(out_beg, 0);
        assert_eq!(out_nb, 3);
        assert_eq!(out, [5.0, 7.0, 9.0]);
        assert_eq!(TA_Shutdown(), RetCode::Success as i32);
    }

    #[test]
    fn ffi_retcode_info_populates_strings() {
        let mut info = TA_RetCodeInfo {
            enumStr: core::ptr::null(),
            infoStr: core::ptr::null(),
        };

        // SAFETY: info points to a valid writable struct.
        unsafe { TA_SetRetCodeInfo(RetCode::BadParam as i32, &mut info) };

        assert!(!info.enumStr.is_null());
        assert!(!info.infoStr.is_null());
    }
}
