#[macro_export]
macro_rules! generate_wrapper_methods {
    ($format:ident, $deserialize_method:ident, $serialize_method:ident, $deserialize_func_str:expr, $serialize_func_str:expr, $deserialize_error:expr, $serialize_error:expr) => {
        impl<T> Wrapper<T>
        where
            T: DeserializeOwned + Serialize,
        {
            pub fn new(data: T, path: &str) -> Self {
                Self {
                    inner: data,
                    path: path.to_string(),
                }
            }

            pub fn load(path: &str, ignore: bool) -> Option<T> {
                match std::fs::read_to_string(path) {
                    Ok(text) => Self::loads(&text, ignore),
                    Err(e) => {
                        if !ignore {
                            tracing::error!(
                                func = "std::fs::read_to_string",
                                path = path,
                                error_tag = ErrorTag::ReadFileError.as_ref(),
                                error_str = e.to_string(),
                            );
                        }
                        None
                    }
                }
            }

            pub fn loads(text: &str, ignore: bool) -> Option<T> {
                match $format::$deserialize_method(text) {
                    Ok(c) => Some(c),
                    Err(e) => {
                        if !ignore {
                            tracing::error!(
                                func = $deserialize_func_str,
                                error_tag = $deserialize_error,
                                error_str = e.to_string(),
                                message = text,
                            );
                        }
                        None
                    }
                }
            }

            pub fn dump(&self, ignore: bool) -> bool {
                Self::dump_data(&self.inner, &self.path, ignore)
            }

            pub fn dump_data(data: &T, path: &str, ignore: bool) -> bool {
                let text = Self::dumps_data(data, ignore);
                if text.is_empty() {
                    return false;
                }

                match std::fs::write(&path, text.as_bytes()) {
                    Ok(_) => true,
                    Err(e) => {
                        if !ignore {
                            tracing::error!(
                                func = "std::fs::write",
                                path = path,
                                error_tag = ErrorTag::WriteFileError.as_ref(),
                                error_str = e.to_string(),
                                message = text,
                            );
                        }
                        false
                    }
                }
            }

            pub fn dumps(&self, ignore: bool) -> String {
                Self::dumps_data(&self.inner, ignore)
            }

            pub fn dumps_data(data: &T, ignore: bool) -> String {
                match $format::$serialize_method(data) {
                    Ok(text) => text,
                    Err(e) => {
                        if !ignore {
                            tracing::error!(
                                func = $serialize_func_str,
                                error_tag = $serialize_error,
                                error_str = e.to_string(),
                            );
                        }
                        String::new()
                    }
                }
            }
        }
    };
}
