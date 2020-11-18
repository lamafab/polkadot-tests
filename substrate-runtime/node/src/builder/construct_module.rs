macro_rules! module {
    (
        #[serde(rename = $module_name:expr)]
        struct $struct:ident;

        enum $enum:ident {
            $(
                #[serde(rename = $func_name:expr)]
                $func:ident {
                    $($func_tt:tt)*
                },
            )*
        }

        impl $struct2:ident {
            fn run($self:ident) -> Result<$ret:ty> $run_body:block
        }
    ) => {
        #[derive(Debug, StructOpt, Serialize, Deserialize)]
        #[serde(rename = $module_name)]
        pub struct $struct {
            #[structopt(subcommand)]
            call: $enum,
        }

        #[derive(Debug, StructOpt, Serialize, Deserialize)]
        pub enum $enum {
            $(
                #[serde(rename = $func_name)]
                $func {
                        $($func_tt)*
                },
            )*
        }

        impl crate::builder::ModuleInfo for $struct {
            fn module_name(&self) -> crate::builder::ModuleName {
                crate::builder::ModuleName::from($module_name)
            }
            fn function_name(&self) -> crate::builder::FunctionName {
                match self.call {
                    $( $enum::$func { .. } => crate::builder::FunctionName::from($func_name), )*
                }
            }
        }

        impl crate::builder::Builder for $struct {
            type Input = $enum;
            type Output = $ret;

            fn run($self) -> crate::Result<Self::Output> $run_body
        }

        // TODO: Delete this
        impl $struct2 {
            pub fn run($self) -> crate::Result<$ret> $run_body
        }
    };
}
