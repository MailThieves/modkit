pub mod camera {
    use std::path::PathBuf;
    use std::process::Command;
    use std::thread::sleep;
    use std::time::Duration;

    use image::{ImageBuffer, RgbImage};
    use log::*;

    use super::super::light::light;
    use super::super::DeviceError;
    use crate::defaults;
    use crate::drivers::hardware_enabled;

    enum FileType {
        Image,
        Video,
    }

    /// Prepares the output dir and generates a file name inside that dir.
    /// Giving FileType::Video will give a path with an .h264 extension,
    /// FileType::Image will give .jpg
    fn get_output_file(file_type: FileType) -> Result<PathBuf, DeviceError> {
        let img_dir = defaults::img_dir();

        trace!("Using {img_dir} as the image location");

        let dir_path: PathBuf = PathBuf::from(img_dir);

        // Make sure the dir exists
        if !dir_path.exists() {
            error!(
                "Path {} doesn't exist. Please pass a valid directory.",
                dir_path.display()
            );

            return Err(DeviceError::IoError(format!(
                "path {} does not exist",
                dir_path.display()
            )));
        }

        // And make sure it's actually a dir
        if !dir_path.is_dir() {
            error!("Path {} is not a directory", dir_path.display());
            return Err(DeviceError::IoError(format!(
                "path {} is not a directory",
                dir_path.display()
            )));
        }

        // Create a file name within the dir
        let mut img_path = PathBuf::from(dir_path);
        match file_type {
            FileType::Image => img_path.push(format!("{}.jpg", chrono::Utc::now().timestamp())),
            FileType::Video => img_path.push(format!("{}.h264", chrono::Utc::now().timestamp())),
        }
        Ok(img_path)
    }

    pub fn capture_still() -> Result<PathBuf, DeviceError> {
        let hardware = hardware_enabled();

        if hardware {
            trace!("Turning light on to capture image");
            light::set(true)?;
            sleep(Duration::from_millis(50));
        }

        let img_path = get_output_file(FileType::Image)?;

        trace!("File path for captured image: {}", img_path.display());

        if hardware {
            let mut args = vec![
                "--drc",
                "high",
                "--width",
                "800",
                "--height",
                "550",
                "--timeout",
                "1",
                "--nopreview",
                "--brightness",
                "50",
                "--ISO",
                "100",
            ];

            if defaults::flip_vertical() {
                args.push("-vf");
            }

            args.push("-o");
            let path_display = format!("{}", img_path.display());
            args.push(&path_display);

            trace!("Image capture command = `raspistill {:?}`", args);

            trace!("Taking picture with raspistill");
            Command::new("raspistill")
                .args(&args)
                .output()
                .expect("Run raspistill command");

            sleep(Duration::from_millis(50));
            trace!("Turning light off after image capture");
            light::set(false)?;
        } else {
            // Generate an image and save it
            let mut img: RgbImage = ImageBuffer::new(50, 50);
            *img.get_pixel_mut(25, 25) = image::Rgb([255, 255, 255]);

            trace!("Writing captured image to {}", img_path.display());
            img.save(&img_path)?;
        }

        // And return the path
        Ok(img_path)
    }

    pub fn capture_video() -> Result<PathBuf, DeviceError> {
        let hardware = hardware_enabled();

        if hardware {
            trace!("Turning light on to capture image");
            light::set(true)?;
            sleep(Duration::from_millis(50));
        }

        let unproc_video_path = get_output_file(FileType::Video)?;
        let mut proc_video_path = unproc_video_path.clone();
        proc_video_path.set_extension("mp4");

        trace!(
            "File path for captured preprocessed video: {}",
            unproc_video_path.display()
        );
        trace!(
            "File path for captured processed video: {}",
            proc_video_path.display()
        );

        if hardware {
            let mut args = vec!["-w", "800", "-h", "550", "-fps", "25", "--nopreview"];

            if defaults::flip_vertical() {
                args.push("-vf")
            }

            args.push("-o");
            let path_display = format!("{}", unproc_video_path.display());
            args.push(&path_display);

            // Capture the video as h264
            Command::new("raspivid")
                .args(&args)
                .output()
                .expect("Run raspivid command");

            trace!("Capture unprocessed .h264 video");
            trace!("Converting with ffmpeg");

            Command::new("ffmpeg")
                .args([
                    "-f",
                    "h264",
                    "-i",
                    &format!("{}", unproc_video_path.display()),
                    "-c:v",
                    "copy",
                    &format!("{}", proc_video_path.display()),
                ])
                .output()
                .expect("Convert .h264 to .mp4");

            trace!("Converted");

            match std::fs::remove_file(&unproc_video_path) {
                Ok(_) => trace!("Removed unprocessed file: {}", unproc_video_path.display()),
                Err(e) => error!("Couldn't remove unprocessed .h264 file: {e}"),
            };

            sleep(Duration::from_millis(50));
            trace!("Turning light off after video capture");
            light::set(false)?;
        }

        Ok(proc_video_path)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_capture_and_place_somewhere() {
        let dir = PathBuf::from("./img");
        if !dir.exists() {
            std::fs::create_dir(&dir).unwrap();
        }
        assert!(dir.exists());

        let file_path_res = camera::capture_still();
        assert!(file_path_res.is_ok());
        let file_path = file_path_res.unwrap();
        assert_eq!(file_path.extension().unwrap(), "jpg");

        std::fs::remove_dir_all("./img").unwrap();
    }
}
