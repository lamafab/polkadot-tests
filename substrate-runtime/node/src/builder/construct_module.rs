macro_rules! module {
    (
        #[$m1:meta]
        #[serde(rename = $module_name:expr)]
        struct $struct:ident {
            $($struct_tt:tt)*
        }

        #[$m2:meta]
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
        #[$m1]
        #[serde(rename = $module_name)]
        pub struct $struct { $($struct_tt)* }

        #[$m2]
        enum $enum {
            $(
                #[serde(rename = $func_name)]
                $func {
                    $($func_tt)*
                },
            )*
        }

        const MODULE: ModuleName = ModuleName::from($module_name);

        impl crate::builder::ModuleInfo for $struct {
            fn module_name(&self) -> &'static ModuleName {
                &MODULE
            }
            fn function_name(&self) -> &'static str {
                match self.call {
                    $( $enum::$func { .. } => &$func_name, )*
                }
            }
        }

        // TODO: Delete this
        impl $struct2 {
            pub fn run($self) -> Result<$ret> $run_body
        }
    };
}