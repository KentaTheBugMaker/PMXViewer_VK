pub mod transpiler {
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    use shaderc::ShaderKind;

    pub struct Transpiler {
        source_text: String,
        source_name: String,
        target: ShaderKind,
    }

    impl Transpiler {
        fn open<P: AsRef<Path>>(path: P) -> Self {
            let mut source_text = String::new();
            File::open(&path).unwrap().read_to_string(&mut source_text).unwrap();
            let extension = path.as_ref().extension().unwrap().to_str().unwrap();
            let target = match extension {
                "fsh" => ShaderKind::Fragment,
                "vsh" => ShaderKind::Vertex,
                "gsh" => ShaderKind::Geometry,
                _ => panic!("Error Invalid extension")
            };
            let source_name = path.as_ref().to_str().unwrap().to_string();
            Self {
                source_text,
                source_name,
                target,
            }
        }
        fn transpile(&self) -> String {
            //Preprocessing
            //Generate InOutTable
            //Generate UniformTable
            //call to_string() for InOutTableEntry
            //call to_string() for uniformTableEntry
            //Other text are passed through
            String::new()
        }
    }

    struct UniformTableEntry {
        set: u32,
        binding: u32,
        types: GLSLTypes,
        name: String,
    }

    struct InOutTableEntry {
        complete: GLSLCompleteMode,
        location: u32,
        direction: GLSLDirection,
        types: GLSLTypes,
        name: String,
    }

    enum GLSLCompleteMode {
        Smooth,
        Flat,
    }

    impl ToString for GLSLCompleteMode {
        fn to_string(&self) -> String {
            let from = match self {
                GLSLCompleteMode::Smooth => { "" }
                GLSLCompleteMode::Flat => { "flat" }
            };
            from.to_string()
        }
    }

    impl ToString for InOutTableEntry {
        fn to_string(&self) -> String {
            let mut entry = String::new();
            entry + &self.complete.to_string() + " " + "layout( location = " + &self.location.to_string() + " ) " + &self.direction.to_string() + " " + &self.types.to_string() + " " + &self.name
        }
    }

    impl ToString for UniformTableEntry {
        fn to_string(&self) -> String {
            let mut entry = String::new();
            entry + "uniform" + " layout (" + " set = " + &self.set.to_string() + " , binding = " + &self.binding.to_string() + " ) " + &self.types.to_string() + " " + &self.name + " ;"
        }
    }

    enum GLSLDirection {
        IN,
        OUT,
        INOUT,
    }

    impl ToString for GLSLDirection {
        fn to_string(&self) -> String {
            let from = match self {
                GLSLDirection::IN => { "in" }
                GLSLDirection::OUT => { "out" }
                GLSLDirection::INOUT => { "inout" }
            };
            from.to_string()
        }
    }

    enum GLSLTypes {
        //scalars
        BOOL,
        INT,
        UINT,
        FLOAT,
        DOUBLE,
        //vectors
        BVEC2,
        BVEC3,
        BVEC4,
        IVEC2,
        IVEC3,
        IVEC4,
        UVEC2,
        UVEC3,
        UVEC4,
        VEC2,
        VEC3,
        VEC4,
        DVEC2,
        DVEC3,
        DVEC4,
        //matrices
        MAT2X2,
        MAT2X3,
        MAT2X4,
        MAT3X2,
        MAT3X3,
        MAT3X4,
        MAT4X2,
        MAT4X3,
        MAT4X4,
    }

    impl ToString for GLSLTypes {
        fn to_string(&self) -> String {
            let from = match self {
                GLSLTypes::BOOL => { "bool" }
                GLSLTypes::INT => { "int" }
                GLSLTypes::UINT => { "uint" }
                GLSLTypes::FLOAT => { "float" }
                GLSLTypes::DOUBLE => { "double" }
                GLSLTypes::BVEC2 => { "bvec2" }
                GLSLTypes::BVEC3 => { "bvec3" }
                GLSLTypes::BVEC4 => { "bvec4" }
                GLSLTypes::IVEC2 => { "ivec2" }
                GLSLTypes::IVEC3 => { "ivec3" }
                GLSLTypes::IVEC4 => { "ivec4" }
                GLSLTypes::UVEC2 => { "uvec2" }
                GLSLTypes::UVEC3 => { "uvec3" }
                GLSLTypes::UVEC4 => { "uvec4" }
                GLSLTypes::VEC2 => { "vec2" }
                GLSLTypes::VEC3 => { "vec3" }
                GLSLTypes::VEC4 => { "vec4" }
                GLSLTypes::DVEC2 => { "dvec2" }
                GLSLTypes::DVEC3 => { "dvec3" }
                GLSLTypes::DVEC4 => { "dvec4" }
                GLSLTypes::MAT2X2 => { "mat2" }
                GLSLTypes::MAT2X3 => { "mat2x3" }
                GLSLTypes::MAT2X4 => { "mat2x4" }
                GLSLTypes::MAT3X2 => { "mat3x2" }
                GLSLTypes::MAT3X3 => { "mat3" }
                GLSLTypes::MAT3X4 => { "mat3x4" }
                GLSLTypes::MAT4X2 => { "mat4x2" }
                GLSLTypes::MAT4X3 => { "mat4x3" }
                GLSLTypes::MAT4X4 => { "mat4" }
            };
            from.to_string()
        }
    }
}