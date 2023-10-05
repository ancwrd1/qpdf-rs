use std::{ffi::CString, path::Path, slice};

use crate::{ObjectStreamMode, QPdf, Result, StreamDataMode, StreamDecodeLevel};

/// Print permissions
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, PartialOrd)]
pub enum PrintPermission {
    #[default]
    Full,
    Low,
    None,
}

impl From<PrintPermission> for qpdf_sys::qpdf_r3_print_e {
    fn from(value: PrintPermission) -> Self {
        match value {
            PrintPermission::Full => qpdf_sys::qpdf_r3_print_e_qpdf_r3p_full,
            PrintPermission::Low => qpdf_sys::qpdf_r3_print_e_qpdf_r3p_low,
            PrintPermission::None => qpdf_sys::qpdf_r3_print_e_qpdf_r3p_none,
        }
    }
}

/// Encryption using RC4 with key length of 40 bits
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct EncryptionParamsR2 {
    pub user_password: String,
    pub owner_password: String,
    pub allow_print: bool,
    pub allow_modify: bool,
    pub allow_extract: bool,
    pub allow_annotate: bool,
}

/// Encryption using RC4 with key length of 128 bits.
/// Minimal PDF version: 1.4.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct EncryptionParamsR3 {
    pub user_password: String,
    pub owner_password: String,
    pub allow_accessibility: bool,
    pub allow_extract: bool,
    pub allow_assemble: bool,
    pub allow_annotate_and_form: bool,
    pub allow_form_filling: bool,
    pub allow_modify_other: bool,
    pub allow_print: PrintPermission,
}

/// Encryption using RC4-128 or AES-256 algorithm and additional flag to encrypt metadata.
/// Minimal PDF version: 1.5.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct EncryptionParamsR4 {
    pub user_password: String,
    pub owner_password: String,
    pub allow_accessibility: bool,
    pub allow_extract: bool,
    pub allow_assemble: bool,
    pub allow_annotate_and_form: bool,
    pub allow_form_filling: bool,
    pub allow_modify_other: bool,
    pub allow_print: PrintPermission,
    pub encrypt_metadata: bool,
    pub use_aes: bool,
}

/// Encryption using AES-256 algorithm and additional flag to encrypt metadata
/// Minimal PDF version: 1.7. Is required for PDF 2.0.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct EncryptionParamsR6 {
    pub user_password: String,
    pub owner_password: String,
    pub allow_accessibility: bool,
    pub allow_extract: bool,
    pub allow_assemble: bool,
    pub allow_annotate_and_form: bool,
    pub allow_form_filling: bool,
    pub allow_modify_other: bool,
    pub allow_print: PrintPermission,
    pub encrypt_metadata: bool,
}

/// Encryption parameters selector
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EncryptionParams {
    /// R2 level, any PDF version
    R2(EncryptionParamsR2),
    /// R3 level, PDF version >= 1.4
    R3(EncryptionParamsR3),
    /// R4 level, PDF version >= 1.5
    R4(EncryptionParamsR4),
    /// R6 level, PDF version >= 1.7
    R6(EncryptionParamsR6),
}

/// PDF writer with several customizable parameters
pub struct QPdfWriter {
    owner: QPdf,
    compress_streams: Option<bool>,
    preserve_unreferenced_objects: Option<bool>,
    normalize_content: Option<bool>,
    preserve_encryption: Option<bool>,
    linearize: Option<bool>,
    static_id: Option<bool>,
    deterministic_id: Option<bool>,
    min_pdf_version: Option<String>,
    force_pdf_version: Option<String>,
    stream_decode_level: Option<StreamDecodeLevel>,
    object_stream_mode: Option<ObjectStreamMode>,
    stream_data_mode: Option<StreamDataMode>,
    encryption_params: Option<EncryptionParams>,
}

impl QPdfWriter {
    pub(crate) fn new(owner: QPdf) -> Self {
        QPdfWriter {
            owner,
            compress_streams: None,
            preserve_unreferenced_objects: None,
            normalize_content: None,
            preserve_encryption: None,
            linearize: None,
            static_id: None,
            deterministic_id: None,
            min_pdf_version: None,
            force_pdf_version: None,
            stream_decode_level: None,
            object_stream_mode: None,
            stream_data_mode: None,
            encryption_params: None,
        }
    }

    fn process_params(&self) -> Result<()> {
        unsafe {
            if let Some(compress_streams) = self.compress_streams {
                qpdf_sys::qpdf_set_compress_streams(self.owner.inner(), compress_streams.into());
            }

            if let Some(preserve_unreferenced_objects) = self.preserve_unreferenced_objects {
                qpdf_sys::qpdf_set_preserve_unreferenced_objects(
                    self.owner.inner(),
                    preserve_unreferenced_objects.into(),
                );
            }

            if let Some(normalize_content) = self.normalize_content {
                qpdf_sys::qpdf_set_content_normalization(self.owner.inner(), normalize_content.into());
            }

            if let Some(preserve_encryption) = self.preserve_encryption {
                qpdf_sys::qpdf_set_preserve_encryption(self.owner.inner(), preserve_encryption.into());
            }

            if let Some(linearize) = self.linearize {
                qpdf_sys::qpdf_set_linearization(self.owner.inner(), linearize.into());
            }

            if let Some(static_id) = self.static_id {
                qpdf_sys::qpdf_set_static_ID(self.owner.inner(), static_id.into());
            }

            if let Some(deterministic_id) = self.deterministic_id {
                qpdf_sys::qpdf_set_deterministic_ID(self.owner.inner(), deterministic_id.into());
            }

            if let Some(stream_decode_level) = self.stream_decode_level {
                qpdf_sys::qpdf_set_decode_level(self.owner.inner(), stream_decode_level.as_qpdf_enum());
            }

            if let Some(object_stream_mode) = self.object_stream_mode {
                qpdf_sys::qpdf_set_object_stream_mode(self.owner.inner(), object_stream_mode.as_qpdf_enum());
            }

            if let Some(stream_data_mode) = self.stream_data_mode {
                qpdf_sys::qpdf_set_stream_data_mode(self.owner.inner(), stream_data_mode.as_qpdf_enum());
            }

            if let Some(ref version) = self.min_pdf_version {
                let version = CString::new(version.as_str())?;
                self.owner
                    .wrap_ffi_call(|| qpdf_sys::qpdf_set_minimum_pdf_version(self.owner.inner(), version.as_ptr()))?;
            }
            if let Some(ref version) = self.force_pdf_version {
                let version = CString::new(version.as_str())?;
                self.owner
                    .wrap_ffi_call(|| qpdf_sys::qpdf_force_pdf_version(self.owner.inner(), version.as_ptr()))?;
            }
            if let Some(ref params) = self.encryption_params {
                self.set_encryption_params(params)?;
            }
        }
        Ok(())
    }

    fn set_encryption_params(&self, params: &EncryptionParams) -> Result<()> {
        match params {
            EncryptionParams::R2(r2) => {
                let user_password = CString::new(r2.user_password.as_str())?;
                let owner_password = CString::new(r2.owner_password.as_str())?;
                unsafe {
                    self.owner.wrap_ffi_call(|| {
                        qpdf_sys::qpdf_set_r2_encryption_parameters(
                            self.owner.inner(),
                            user_password.as_ptr(),
                            owner_password.as_ptr(),
                            r2.allow_print.into(),
                            r2.allow_modify.into(),
                            r2.allow_extract.into(),
                            r2.allow_annotate.into(),
                        )
                    })?;
                }
            }
            EncryptionParams::R3(r3) => {
                let user_password = CString::new(r3.user_password.as_str())?;
                let owner_password = CString::new(r3.owner_password.as_str())?;
                unsafe {
                    self.owner.wrap_ffi_call(|| {
                        qpdf_sys::qpdf_set_r3_encryption_parameters2(
                            self.owner.inner(),
                            user_password.as_ptr(),
                            owner_password.as_ptr(),
                            r3.allow_accessibility.into(),
                            r3.allow_extract.into(),
                            r3.allow_assemble.into(),
                            r3.allow_annotate_and_form.into(),
                            r3.allow_form_filling.into(),
                            r3.allow_modify_other.into(),
                            r3.allow_print.into(),
                        )
                    })?;
                }
            }
            EncryptionParams::R4(r4) => {
                let user_password = CString::new(r4.user_password.as_str())?;
                let owner_password = CString::new(r4.owner_password.as_str())?;
                unsafe {
                    self.owner.wrap_ffi_call(|| {
                        qpdf_sys::qpdf_set_r4_encryption_parameters2(
                            self.owner.inner(),
                            user_password.as_ptr(),
                            owner_password.as_ptr(),
                            r4.allow_accessibility.into(),
                            r4.allow_extract.into(),
                            r4.allow_assemble.into(),
                            r4.allow_annotate_and_form.into(),
                            r4.allow_form_filling.into(),
                            r4.allow_modify_other.into(),
                            r4.allow_print.into(),
                            r4.encrypt_metadata.into(),
                            r4.use_aes.into(),
                        )
                    })?;
                }
            }
            EncryptionParams::R6(r6) => {
                let user_password = CString::new(r6.user_password.as_str())?;
                let owner_password = CString::new(r6.owner_password.as_str())?;
                unsafe {
                    self.owner.wrap_ffi_call(|| {
                        qpdf_sys::qpdf_set_r6_encryption_parameters2(
                            self.owner.inner(),
                            user_password.as_ptr(),
                            owner_password.as_ptr(),
                            r6.allow_accessibility.into(),
                            r6.allow_extract.into(),
                            r6.allow_assemble.into(),
                            r6.allow_annotate_and_form.into(),
                            r6.allow_form_filling.into(),
                            r6.allow_modify_other.into(),
                            r6.allow_print.into(),
                            r6.encrypt_metadata.into(),
                        )
                    })?;
                }
            }
        }
        Ok(())
    }

    /// Write PDF to a file
    pub fn write<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let filename = CString::new(path.as_ref().to_string_lossy().as_ref())?;

        let inner = self.owner.inner();

        self.owner
            .wrap_ffi_call(|| unsafe { qpdf_sys::qpdf_init_write(inner, filename.as_ptr()) })?;

        self.process_params()?;

        self.owner.wrap_ffi_call(|| unsafe { qpdf_sys::qpdf_write(inner) })
    }

    /// Write PDF to a memory and return it in a Vec
    pub fn write_to_memory(&self) -> Result<Vec<u8>> {
        let inner = self.owner.inner();
        self.owner
            .wrap_ffi_call(|| unsafe { qpdf_sys::qpdf_init_write_memory(inner) })?;

        self.process_params()?;

        self.owner.wrap_ffi_call(|| unsafe { qpdf_sys::qpdf_write(inner) })?;

        let buffer = unsafe { qpdf_sys::qpdf_get_buffer(inner) };
        let buffer_len = unsafe { qpdf_sys::qpdf_get_buffer_length(inner) };

        unsafe { Ok(slice::from_raw_parts(buffer, buffer_len as _).to_vec()) }
    }

    /// Enable or disable stream compression
    pub fn compress_streams(&mut self, flag: bool) -> &mut Self {
        self.compress_streams = Some(flag);
        self
    }

    /// Set minimum PDF version
    pub fn minimum_pdf_version(&mut self, version: &str) -> &mut Self {
        self.min_pdf_version = Some(version.to_owned());
        self
    }

    /// Force a specific PDF version
    pub fn force_pdf_version(&mut self, version: &str) -> &mut Self {
        self.force_pdf_version = Some(version.to_owned());
        self
    }

    /// Set stream decode level
    pub fn stream_decode_level(&mut self, level: StreamDecodeLevel) -> &mut Self {
        self.stream_decode_level = Some(level);
        self
    }

    /// Set object stream mode
    pub fn object_stream_mode(&mut self, mode: ObjectStreamMode) -> &mut Self {
        self.object_stream_mode = Some(mode);
        self
    }

    /// Set stream data mode
    pub fn stream_data_mode(&mut self, mode: StreamDataMode) -> &mut Self {
        self.stream_data_mode = Some(mode);
        self
    }

    /// Set a flag indicating whether to preserve the unreferenced objects
    pub fn preserve_unreferenced_objects(&mut self, flag: bool) -> &mut Self {
        self.preserve_unreferenced_objects = Some(flag);
        self
    }

    /// Set a flag indicating whether to normalized contents
    pub fn normalize_content(&mut self, flag: bool) -> &mut Self {
        self.normalize_content = Some(flag);
        self
    }

    /// Preserve or remove encryption
    pub fn preserve_encryption(&mut self, flag: bool) -> &mut Self {
        self.preserve_encryption = Some(flag);
        self
    }

    /// Enable or disable linearization
    pub fn linearize(&mut self, flag: bool) -> &mut Self {
        self.linearize = Some(flag);
        self
    }

    // Enable or disable static ID
    pub fn static_id(&mut self, flag: bool) -> &mut Self {
        self.static_id = Some(flag);
        self
    }

    // Enable or disable deterministic ID
    pub fn deterministic_id(&mut self, flag: bool) -> &mut Self {
        self.deterministic_id = Some(flag);
        self
    }

    // Set encryption parameters
    pub fn encryption_params(&mut self, params: EncryptionParams) -> &mut Self {
        self.encryption_params = Some(params);
        self
    }
}
