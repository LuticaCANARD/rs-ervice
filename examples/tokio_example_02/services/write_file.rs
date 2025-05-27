use rs_ervice::RSContextService;
use std::any::Any;
use std::fs::File;
use std::io::{self, Write};


pub struct WriteFileService{
    now_file_path: String,
    cl_lf: bool,
    stream: Option<Box<dyn Write>>
}

impl WriteFileService{

    pub fn open_file(&mut self, file_path: &str) -> io::Result<()> {
        self.now_file_path = file_path.to_string();
        let mut file = File::create(&self.now_file_path)?;
        if self.cl_lf {
            writeln!(file, "File opened at: {}", self.now_file_path)?;
        } else {
            write!(file, "File opened at: {}", self.now_file_path)?;
        }
        Ok(())
    }

}

impl RSContextService for WriteFileService{
    async fn on_register_crate_instance() -> Self {
        WriteFileService {
            now_file_path: String::new(),
            cl_lf: false,
            stream: None,
        }
    }

    async fn on_service_created(&mut self, service_builder: &rs_ervice::RSContextBuilder) -> Result<(), rs_ervice::RsServiceError> {
        // 서비스가 등록될 때 호출되는 메서드
        println!("WriteFileService registered successfully!");
        Ok(())
    }

    async fn on_all_services_built(&self, context: &rs_ervice::RSContext) -> Result<(), rs_ervice::RsServiceError> {
        // 모든 서비스가 빌드된 후 호출되는 메서드
        println!("All services built successfully in context: {:?}", context.type_id());
        Ok(())
    }
}