use std::{
    ffi::{c_char, c_int, c_long, CStr},
    fmt,
    io::SeekFrom,
    ops::Deref,
    ptr::{null_mut, NonNull},
};

use bitflags::bitflags;
use libc::{c_void, SEEK_CUR, SEEK_END, SEEK_SET};

pub use cdparanoia_sys;
pub use cdparanoia_sys::{CD_FRAMESAMPLES, CD_FRAMESIZE, CD_FRAMESIZE_RAW, CD_FRAMEWORDS};

pub fn cdda_version() -> &'static CStr {
    unsafe { CStr::from_ptr(cdparanoia_sys::cdda_version()) }
}

pub fn paranoia_version() -> &'static CStr {
    unsafe { CStr::from_ptr(cdparanoia_sys::paranoia_version()) }
}

#[repr(u32)]
pub enum Verbosity {
    ForgetIt = cdparanoia_sys::CDDA_MESSAGE_FORGETIT,
    PrintIt = cdparanoia_sys::CDDA_MESSAGE_PRINTIT,
    LogIt = cdparanoia_sys::CDDA_MESSAGE_LOGIT,
}

#[derive(Debug)]
pub struct Error {
    raw: c_int,
}

impl Error {
    pub fn from_raw(raw: c_int) -> Result<(), Self> {
        if raw >= 0 {
            Ok(())
        } else {
            Err(Self { raw })
        }
    }

    pub fn from_raw_long(raw: c_long) -> Result<(), Self> {
        if raw >= 0 {
            Ok(())
        } else {
            Err(Self { raw: raw as _ })
        }
    }

    pub fn as_raw(&self) -> c_int {
        self.raw
    }

    pub fn code(&self) -> Option<ErrorCode> {
        ErrorCode::from_raw(self.raw)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.code() {
            Some(code) => write!(f, "{}", code),
            None => write!(f, "Unknown error code {}", self.raw),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, Copy, PartialEq, displaydoc::Display)]
#[non_exhaustive]
pub enum ErrorCode {
    /// 001: Unable to set CDROM to read audio mode
    NoReadMode,

    /// 002: Unable to read table of contents lead-out
    NoTocLeadout,

    /// 003: CDROM reporting illegal number of tracks
    IllegalTrackCount,

    /// 004: Unable to read table of contents header
    NoTocHeader,

    /// 005: Unable to read table of contents entry
    NoTocEntry,

    /// 006: Could not read any data from drive
    CannotReadAnyData,

    /// 007: Unknown, unrecoverable error reading data
    UnknownReadError,

    /// 008: Unable to identify CDROM model
    NoCdromModel,

    /// 009: CDROM reporting illegal table of contents
    IllegalToc,

    /// 100: Interface not supported
    InterfaceNotSupported,

    /// 102: Permission denied on cdrom (ioctl) device
    PermissionDenied,

    /// 300: Kernel memory error
    KernelMemoryError,

    /// 400: Device not open
    DeviceNotOpen,

    /// 401: Invalid track number
    InvalidTrackNumber,

    /// 403: No audio tracks on disc
    NoAudioTracks,

    /// 404: No medium present
    NoMediumPresent,

    /// 405: Option not supported by drive
    OptionNotSupported,
}

impl ErrorCode {
    pub fn from_raw(raw: c_int) -> Option<Self> {
        match raw.wrapping_abs() {
            1 => Some(Self::NoReadMode),
            2 => Some(Self::NoTocLeadout),
            3 => Some(Self::IllegalTrackCount),
            4 => Some(Self::NoTocHeader),
            5 => Some(Self::NoTocEntry),
            6 => Some(Self::CannotReadAnyData),
            7 => Some(Self::UnknownReadError),
            8 => Some(Self::NoCdromModel),
            9 => Some(Self::IllegalToc),
            100 => Some(Self::InterfaceNotSupported),
            102 => Some(Self::PermissionDenied),
            300 => Some(Self::KernelMemoryError),
            400 => Some(Self::DeviceNotOpen),
            401 => Some(Self::InvalidTrackNumber),
            403 => Some(Self::NoAudioTracks),
            404 => Some(Self::NoMediumPresent),
            405 => Some(Self::OptionNotSupported),
            _ => None,
        }
    }
}

pub struct CddaString {
    raw: NonNull<c_char>,
}

impl CddaString {
    pub unsafe fn from_raw(raw: *mut c_char) -> Option<Self> {
        NonNull::new(raw).map(|raw| Self { raw })
    }

    pub fn as_c_str(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.raw.as_ptr() as *const c_char) }
    }
}

impl Deref for CddaString {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        self.as_c_str()
    }
}

impl Drop for CddaString {
    fn drop(&mut self) {
        unsafe { libc::free(self.raw.as_ptr() as *mut c_void) }
    }
}

unsafe impl Send for CddaString {}
unsafe impl Sync for CddaString {}

bitflags! {
    pub struct ParanoiaMode : u32 {
        const FRAGMENT = cdparanoia_sys::PARANOIA_MODE_FRAGMENT;
        const NEVERSKIP = cdparanoia_sys::PARANOIA_MODE_NEVERSKIP;
        const OVERLAP = cdparanoia_sys::PARANOIA_MODE_OVERLAP;
        const REPAIR = cdparanoia_sys::PARANOIA_MODE_REPAIR;
        const SCRATCH = cdparanoia_sys::PARANOIA_MODE_SCRATCH;
        const VERIFY = cdparanoia_sys::PARANOIA_MODE_VERIFY;

        const FULL = cdparanoia_sys::PARANOIA_MODE_FULL;
        const DISABLE = cdparanoia_sys::PARANOIA_MODE_DISABLE;
    }
}

pub struct CdromDrive {
    raw: NonNull<cdparanoia_sys::cdrom_drive>,
}

impl CdromDrive {
    pub unsafe fn from_raw(raw: *mut cdparanoia_sys::cdrom_drive) -> Option<Self> {
        NonNull::new(raw).map(|raw| Self { raw })
    }

    pub fn as_raw(&self) -> *mut cdparanoia_sys::cdrom_drive {
        self.raw.as_ptr()
    }

    pub fn into_raw(self) -> *mut cdparanoia_sys::cdrom_drive {
        let raw = self.as_raw();
        // Avoid dropping self, otherwise cdda_close will be called in Drop
        std::mem::forget(self);
        raw
    }

    pub fn find_a_cdrom(verbosity: Verbosity) -> Option<Self> {
        // TODO messages output
        unsafe {
            Self::from_raw(cdparanoia_sys::cdda_find_a_cdrom(
                verbosity as c_int,
                null_mut(),
            ))
        }
    }

    pub fn identify(device: &CStr, verbosity: Verbosity) -> Option<Self> {
        // TODO messages output
        unsafe {
            Self::from_raw(cdparanoia_sys::cdda_identify(
                device.as_ptr(),
                verbosity as c_int,
                null_mut(),
            ))
        }
    }

    pub fn identify_scsi(
        generic_device: &CStr,
        ioctl_device: &CStr,
        verbosity: Verbosity,
    ) -> Option<Self> {
        // TODO messages output
        unsafe {
            Self::from_raw(cdparanoia_sys::cdda_identify_scsi(
                generic_device.as_ptr(),
                ioctl_device.as_ptr(),
                verbosity as c_int,
                null_mut(),
            ))
        }
    }

    pub fn identify_cooked(device: &CStr, verbosity: Verbosity) -> Option<Self> {
        // TODO messages output
        unsafe {
            Self::from_raw(cdparanoia_sys::cdda_identify_cooked(
                device.as_ptr(),
                verbosity as c_int,
                null_mut(),
            ))
        }
    }

    pub fn set_verbosity(&self, error_verbosity: Verbosity, message_verbosity: Verbosity) {
        unsafe {
            cdparanoia_sys::cdda_verbose_set(
                self.raw.as_ptr(),
                error_verbosity as c_int,
                message_verbosity as c_int,
            );
        }
    }

    pub fn open(&self) -> Result<(), Error> {
        Error::from_raw(unsafe { cdparanoia_sys::cdda_open(self.raw.as_ptr()) })
    }

    pub fn set_speed(&self, speed: i32) -> Result<(), Error> {
        Error::from_raw(unsafe {
            cdparanoia_sys::cdda_speed_set(self.raw.as_ptr(), speed.try_into().unwrap())
        })
    }

    pub fn disc_first_sector(&self) -> Result<u64, Error> {
        let result = unsafe { cdparanoia_sys::cdda_disc_firstsector(self.raw.as_ptr()) };
        Error::from_raw_long(result)?;
        Ok(result.try_into().unwrap())
    }

    pub fn track_first_sector(&self, track: u32) -> Result<u64, Error> {
        let result = unsafe {
            cdparanoia_sys::cdda_track_firstsector(self.raw.as_ptr(), track.try_into().unwrap())
        };
        Error::from_raw_long(result)?;
        Ok(result.try_into().unwrap())
    }

    pub fn track_last_sector(&self, track: u32) -> Result<u64, Error> {
        let result = unsafe {
            cdparanoia_sys::cdda_track_lastsector(self.raw.as_ptr(), track.try_into().unwrap())
        };
        Error::from_raw_long(result)?;
        Ok(result.try_into().unwrap())
    }

    pub fn sector_get_track(&self, sector: u64) -> Result<u32, Error> {
        let result = unsafe {
            cdparanoia_sys::cdda_sector_gettrack(self.raw.as_ptr(), sector.try_into().unwrap())
        };
        Error::from_raw(result)?;
        Ok(result.try_into().unwrap())
    }

    pub fn tracks(&self) -> Result<u32, Error> {
        let result = unsafe { cdparanoia_sys::cdda_tracks(self.raw.as_ptr()) };
        Error::from_raw_long(result)?;
        Ok(result.try_into().unwrap())
    }

    pub fn track_channels(&self, track: u32) -> Result<u32, Error> {
        let result = unsafe {
            cdparanoia_sys::cdda_track_channels(self.raw.as_ptr(), track.try_into().unwrap())
        };
        Error::from_raw(result)?;
        Ok(result.try_into().unwrap())
    }

    pub fn track_audiop(&self, track: u32) -> Result<bool, Error> {
        let result = unsafe {
            cdparanoia_sys::cdda_track_audiop(self.raw.as_ptr(), track.try_into().unwrap())
        };
        Error::from_raw(result)?;
        Ok(result != 0)
    }

    pub fn track_copyp(&self, track: u32) -> Result<bool, Error> {
        let result = unsafe {
            cdparanoia_sys::cdda_track_copyp(self.raw.as_ptr(), track.try_into().unwrap())
        };
        Error::from_raw(result)?;
        Ok(result != 0)
    }

    pub fn track_preemp(&self, track: u32) -> Result<bool, Error> {
        let result = unsafe {
            cdparanoia_sys::cdda_track_preemp(self.raw.as_ptr(), track.try_into().unwrap())
        };
        Error::from_raw(result)?;
        Ok(result != 0)
    }

    pub fn messages(&self) -> Option<CddaString> {
        unsafe { CddaString::from_raw(cdparanoia_sys::cdda_messages(self.raw.as_ptr())) }
    }

    pub fn errors(&self) -> Option<CddaString> {
        unsafe { CddaString::from_raw(cdparanoia_sys::cdda_errors(self.raw.as_ptr())) }
    }
}

impl Drop for CdromDrive {
    fn drop(&mut self) {
        unsafe {
            cdparanoia_sys::cdda_close(self.raw.as_ptr());
        }
    }
}

pub struct CdromParanoia {
    drive: CdromDrive,
    raw: NonNull<cdparanoia_sys::cdrom_paranoia>,
}

impl CdromParanoia {
    pub unsafe fn from_raw(drive: CdromDrive, raw: *mut cdparanoia_sys::cdrom_paranoia) -> Self {
        Self {
            drive,
            raw: NonNull::new(raw).unwrap(),
        }
    }

    pub fn as_raw(&self) -> *mut cdparanoia_sys::cdrom_paranoia {
        self.raw.as_ptr()
    }

    pub fn into_raw(self) -> (CdromDrive, *mut cdparanoia_sys::cdrom_paranoia) {
        // Need to reconstruct the drive, it is not possible to move out of self.
        let raw_drive = self.drive.as_raw();
        let raw = self.as_raw();

        // Avoid dropping self, otherwise cdda_close and paranoia_free will be
        // called in Drop
        std::mem::forget(self);

        (unsafe { CdromDrive::from_raw(raw_drive).unwrap() }, raw)
    }

    pub fn init(drive: CdromDrive) -> Self {
        let raw = unsafe { cdparanoia_sys::paranoia_init(drive.as_raw()) };
        unsafe { Self::from_raw(drive, raw) }
    }

    pub fn drive(&self) -> &CdromDrive {
        &self.drive
    }

    pub fn set_mode(&self, mode: ParanoiaMode) {
        unsafe {
            cdparanoia_sys::paranoia_modeset(self.raw.as_ptr(), mode.bits().try_into().unwrap());
        }
    }

    pub fn set_overlap(&self, overlap: i64) {
        unsafe {
            cdparanoia_sys::paranoia_overlapset(self.raw.as_ptr(), overlap.try_into().unwrap());
        }
    }

    pub fn seek(&self, pos: SeekFrom) -> Result<u64, Error> {
        let (mode, index): (c_int, c_long) = match pos {
            SeekFrom::Start(x) => (SEEK_SET, x.try_into().unwrap()),
            SeekFrom::End(x) => (SEEK_END, x.try_into().unwrap()),
            SeekFrom::Current(x) => (SEEK_CUR, x.try_into().unwrap()),
        };

        let result = unsafe { cdparanoia_sys::paranoia_seek(self.raw.as_ptr(), index, mode) };
        Error::from_raw_long(result)?;
        Ok(result.try_into().unwrap())
    }

    /// Reads the next sector of audio data and returns a full sector of
    /// verified samples (1176 samples, 2352 bytes).
    pub fn read_limited(
        &mut self,
        callback: extern "C" fn(c_long, c_int),
        max_retries: u32,
    ) -> &[i16; CD_FRAMEWORDS as usize] {
        let ptr = unsafe {
            cdparanoia_sys::paranoia_read_limited(
                self.raw.as_ptr(),
                Some(callback),
                max_retries.try_into().unwrap(),
            )
        };
        unsafe { &*(ptr as *const [i16; CD_FRAMEWORDS as usize]) }
    }

    /// Reads the next sector of audio data and returns a full sector of
    /// verified samples (1176 samples, 2352 bytes).
    ///
    /// Identical to `read_limited` with `max_retries = 20`.
    pub fn read(
        &mut self,
        callback: extern "C" fn(c_long, c_int),
    ) -> &[i16; CD_FRAMEWORDS as usize] {
        let ptr = unsafe { cdparanoia_sys::paranoia_read(self.raw.as_ptr(), Some(callback)) };
        unsafe { &*(ptr as *const [i16; CD_FRAMEWORDS as usize]) }
    }
}

impl Drop for CdromParanoia {
    fn drop(&mut self) {
        unsafe { cdparanoia_sys::paranoia_free(self.raw.as_ptr()) }
    }
}
