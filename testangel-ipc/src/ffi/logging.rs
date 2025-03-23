#[derive(Copy, Clone)]
#[repr(C)]
pub enum ta_logging_level {
    TA_LOG_TRACE = 0,
    TA_LOG_DEBUG = 1,
    TA_LOG_INFO = 2,
    TA_LOG_WARN = 3,
    TA_LOG_ERROR = 4,
}
