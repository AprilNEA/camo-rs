/// Default allowed image content types
pub const IMAGE_TYPES: &[&str] = &[
    "image/bmp",
    "image/cgm",
    "image/g3fax",
    "image/gif",
    "image/ief",
    "image/jp2",
    "image/jpeg",
    "image/jpg",
    "image/pict",
    "image/png",
    "image/prs.btif",
    "image/svg+xml",
    "image/tiff",
    "image/vnd.adobe.photoshop",
    "image/vnd.djvu",
    "image/vnd.dwg",
    "image/vnd.dxf",
    "image/vnd.fastbidsheet",
    "image/vnd.fpx",
    "image/vnd.fst",
    "image/vnd.fujixerox.edmics-mmr",
    "image/vnd.fujixerox.edmics-rlc",
    "image/vnd.microsoft.icon",
    "image/vnd.ms-modi",
    "image/vnd.net-fpx",
    "image/vnd.wap.wbmp",
    "image/vnd.xiff",
    "image/webp",
    "image/avif",
    "image/heic",
    "image/heif",
    "image/x-cmu-raster",
    "image/x-cmx",
    "image/x-icon",
    "image/x-macpaint",
    "image/x-pcx",
    "image/x-pict",
    "image/x-portable-anymap",
    "image/x-portable-bitmap",
    "image/x-portable-graymap",
    "image/x-portable-pixmap",
    "image/x-quicktime",
    "image/x-rgb",
    "image/x-xbitmap",
    "image/x-xpixmap",
    "image/x-xwindowdump",
];

/// Video content types
pub const VIDEO_TYPES: &[&str] = &[
    "video/mp4",
    "video/webm",
    "video/ogg",
    "video/quicktime",
    "video/x-msvideo",
];

/// Audio content types
pub const AUDIO_TYPES: &[&str] = &[
    "audio/mpeg",
    "audio/ogg",
    "audio/wav",
    "audio/webm",
    "audio/flac",
];

/// Check if a content type is an allowed image type
#[allow(dead_code)]
pub fn is_allowed_image_type(content_type: &str) -> bool {
    let ct_lower = content_type.to_lowercase();
    let mime_type = ct_lower.split(';').next().unwrap_or("").trim();
    IMAGE_TYPES.iter().any(|allowed| *allowed == mime_type)
}

/// Check if content type is allowed (with optional video/audio support)
#[allow(dead_code)]
pub fn is_allowed_content_type(content_type: &str, allow_video: bool, allow_audio: bool) -> bool {
    let ct_lower = content_type.to_lowercase();
    let mime_type = ct_lower.split(';').next().unwrap_or("").trim();

    if IMAGE_TYPES.iter().any(|t| *t == mime_type) {
        return true;
    }

    if allow_video && VIDEO_TYPES.iter().any(|t| *t == mime_type) {
        return true;
    }

    if allow_audio && AUDIO_TYPES.iter().any(|t| *t == mime_type) {
        return true;
    }

    false
}
