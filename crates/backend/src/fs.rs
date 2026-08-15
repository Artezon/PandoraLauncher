use std::path::Path;

#[cfg(unix)]
pub struct FileMetadata(std::fs::Metadata);

#[cfg(windows)]
pub struct FileMetadata {
    number_of_links: u32,
    low_precision_id: (u32, u32, u32),
    high_precision_id: Option<(u64, [u8; 16])>,
}

#[cfg(unix)]
impl FileMetadata {
    pub fn new(path: &Path) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(path)?;
        if metadata.is_dir() {
            return Err(std::io::ErrorKind::IsADirectory.into());
        }
        Ok(Self(metadata))
    }

    pub fn is_same(&self, other: &FileMetadata) -> bool {
        use std::os::unix::fs::MetadataExt;
        self.0.dev() == other.0.dev() && self.0.ino() == other.0.ino()
    }

    pub fn number_of_links(&self) -> u64 {
        use std::os::unix::fs::MetadataExt;
        self.0.nlink()
    }
}

#[cfg(windows)]
impl FileMetadata {
    pub fn new(path: &Path) -> std::io::Result<Self> {
        use std::os::windows::io::AsRawHandle;
        use windows::Win32::Storage::FileSystem::FILE_ID_INFO;

        let file = std::fs::OpenOptions::new().open(a)?;
        let handle = file.as_raw_handle() as windows::Win32::Foundation::HANDLE;

        let file_info: BY_HANDLE_FILE_INFORMATION = Default::default();

        let success = unsafe {
            windows::Win32::Storage::FileSystem::GetFileInformationByHandle(
                handle,
                &mut file_info as *mut _,
            )
        };

        if !success {
            return std::io::Result::Err(std::io::Error::last_os_error());
        }

        let mut result = Self {
            number_of_links: file_info.nNumberOfLinks,
            low_precision_id: (file_info.dwVolumeSerialNumber, file_info.nFileIndexHigh, file_info.nFileIndexLow),
            high_precision_id: None,
        };

        let file_id_info: FILE_ID_INFO = Default::default();
        let success = unsafe {
            windows::Win32::Storage::FileSystem::GetFileInformationByHandleEx(
                handle,
                windows::Win32::Storage::FileSystem::FileIdInfo,
                &mut file_id_info as *mut _,
                std::mem::size_of::<FILE_ID_INFO>() as u32,
            )
        };
        if success {
            result.high_precision_id = Some((file_id_info.VolumeSerialNumber, file_id_info.FileId.Identifier));
        }
    }

    pub fn is_same(&self, other: &FileMetadata) -> bool {
        self.low_precision_id == other.low_precision_id && self.high_precision_id == other.high_precision_id
    }

    pub fn number_of_links(&self) -> u64 {
        self.number_of_links as u64
    }
}
