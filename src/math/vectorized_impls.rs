#[macro_export]
macro_rules! op_impl {
    ($name:ident, $op:ident, $op_name:ident, $axis_one:ident, $axis_two:ident, $axis_three:ident) => {
        impl $op<$name> for $name {
            type Output = $name;
            fn $op_name(self, rhs: Self) -> Self::Output {
                $name {
                    $axis_one: self.$axis_one.$op_name(rhs.$axis_one),
                    $axis_two: self.$axis_two.$op_name(rhs.$axis_two),
                    $axis_three: self.$axis_three.$op_name(rhs.$axis_three),
                }
            }
        }

        impl $op<&$name> for &$name {
            type Output = $name;
            fn $op_name(self, rhs: &$name) -> Self::Output {
                $name {
                    $axis_one: self.$axis_one.$op_name(rhs.$axis_one),
                    $axis_two: self.$axis_two.$op_name(rhs.$axis_two),
                    $axis_three: self.$axis_three.$op_name(rhs.$axis_three),
                }
            }
        }

        impl $op<$name> for &$name {
            type Output = $name;
            fn $op_name(self, rhs: $name) -> Self::Output {
                $name {
                    $axis_one: self.$axis_one.$op_name(rhs.$axis_one),
                    $axis_two: self.$axis_two.$op_name(rhs.$axis_two),
                    $axis_three: self.$axis_three.$op_name(rhs.$axis_three),
                }
            }
        }

        impl $op<&$name> for $name {
            type Output = $name;
            fn $op_name(self, rhs: &$name) -> Self::Output {
                $name {
                    $axis_one: self.$axis_one.$op_name(rhs.$axis_one),
                    $axis_two: self.$axis_two.$op_name(rhs.$axis_two),
                    $axis_three: self.$axis_three.$op_name(rhs.$axis_three),
                }
            }
        }
    };
    ($name:ident, $op:ident, $op_name:ident, $axis_one:ident, $axis_two:ident, $axis_three:ident, $axis_four:ident) => {
        impl $op<$name> for $name {
            type Output = $name;
            fn $op_name(self, rhs: Self) -> Self::Output {
                $name {
                    $axis_one: self.$axis_one.$op_name(rhs.$axis_one),
                    $axis_two: self.$axis_two.$op_name(rhs.$axis_two),
                    $axis_three: self.$axis_three.$op_name(rhs.$axis_three),
                    $axis_four: self.$axis_four.$op_name(rhs.$axis_four),
                }
            }
        }

        impl $op<&$name> for &$name {
            type Output = $name;
            fn $op_name(self, rhs: &$name) -> Self::Output {
                $name {
                    $axis_one: self.$axis_one.$op_name(rhs.$axis_one),
                    $axis_two: self.$axis_two.$op_name(rhs.$axis_two),
                    $axis_three: self.$axis_three.$op_name(rhs.$axis_three),
                    $axis_four: self.$axis_three.$op_name(rhs.$axis_four),
                }
            }
        }

        impl $op<$name> for &$name {
            type Output = $name;
            fn $op_name(self, rhs: $name) -> Self::Output {
                $name {
                    $axis_one: self.$axis_one.$op_name(rhs.$axis_one),
                    $axis_two: self.$axis_two.$op_name(rhs.$axis_two),
                    $axis_three: self.$axis_three.$op_name(rhs.$axis_three),
                    $axis_four: self.$axis_three.$op_name(rhs.$axis_four),
                }
            }
        }

        impl $op<&$name> for $name {
            type Output = $name;
            fn $op_name(self, rhs: &$name) -> Self::Output {
                $name {
                    $axis_one: self.$axis_one.$op_name(rhs.$axis_one),
                    $axis_two: self.$axis_two.$op_name(rhs.$axis_two),
                    $axis_three: self.$axis_three.$op_name(rhs.$axis_three),
                    $axis_four: self.$axis_three.$op_name(rhs.$axis_four),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! op_assign_impl {
    ($name:ident, $op:ident, $op_name:ident, $axis_one:ident, $axis_two:ident, $axis_three:ident) => {
        impl $op<$name> for $name {
            fn $op_name(&mut self, rhs: Self) {
                self.$axis_one.$op_name(rhs.$axis_one);
                self.$axis_two.$op_name(rhs.$axis_two);
                self.$axis_three.$op_name(rhs.$axis_three);
            }
        }

        impl $op<&$name> for $name {
            fn $op_name(&mut self, rhs: &$name) {
                self.$axis_one.$op_name(rhs.$axis_one);
                self.$axis_two.$op_name(rhs.$axis_two);
                self.$axis_three.$op_name(rhs.$axis_three);
            }
        }
    };
    ($name:ident, $op:ident, $op_name:ident, $axis_one:ident, $axis_two:ident, $axis_three:ident, $axis_four:ident) => {
        impl $op<$name> for $name {
            fn $op_name(&mut self, rhs: Self) {
                self.$axis_one.$op_name(rhs.$axis_one);
                self.$axis_two.$op_name(rhs.$axis_two);
                self.$axis_three.$op_name(rhs.$axis_three);
                self.$axis_four.$op_name(rhs.$axis_four);
            }
        }

        impl $op<&$name> for $name {
            fn $op_name(&mut self, rhs: &$name) {
                self.$axis_one.$op_name(rhs.$axis_one);
                self.$axis_two.$op_name(rhs.$axis_two);
                self.$axis_three.$op_name(rhs.$axis_three);
                self.$axis_four.$op_name(rhs.$axis_four);
            }
        }
    };
}

#[macro_export]
macro_rules! op_scalar_impl {
    ($name:ident, $scalar:ident, $op:ident, $op_name:ident, $axis_one:ident, $axis_two:ident, $axis_three:ident) => {
        impl $op<$scalar> for $name {
            type Output = $name;
            fn $op_name(self, rhs: $scalar) -> Self::Output {
                $name {
                    $axis_one: self.$axis_one.$op_name(rhs),
                    $axis_two: self.$axis_two.$op_name(rhs),
                    $axis_three: self.$axis_three.$op_name(rhs),
                }
            }
        }

        impl $op<$name> for $scalar {
            type Output = $name;
            fn $op_name(self, rhs: $name) -> Self::Output {
                $name {
                    $axis_one: self.$op_name(rhs.$axis_one),
                    $axis_two: self.$op_name(rhs.$axis_two),
                    $axis_three: self.$op_name(rhs.$axis_three),
                }
            }
        }

        impl $op<$scalar> for &$name {
            type Output = $name;
            fn $op_name(self, rhs: $scalar) -> Self::Output {
                $name {
                    $axis_one: self.$axis_one.$op_name(rhs),
                    $axis_two: self.$axis_two.$op_name(rhs),
                    $axis_three: self.$axis_three.$op_name(rhs),
                }
            }
        }

        impl $op<&$name> for $scalar {
            type Output = $name;
            fn $op_name(self, rhs: &$name) -> Self::Output {
                $name {
                    $axis_one: self.$op_name(rhs.$axis_one),
                    $axis_two: self.$op_name(rhs.$axis_two),
                    $axis_three: self.$op_name(rhs.$axis_three),
                }
            }
        }
    };
    ($name:ident, $scalar:ident, $op:ident, $op_name:ident, $axis_one:ident, $axis_two:ident, $axis_three:ident, $axis_four:ident) => {
        impl $op<$scalar> for $name {
            type Output = $name;
            fn $op_name(self, rhs: $scalar) -> Self::Output {
                $name {
                    $axis_one: self.$axis_one.$op_name(rhs),
                    $axis_two: self.$axis_two.$op_name(rhs),
                    $axis_three: self.$axis_three.$op_name(rhs),
                    $axis_four: self.$axis_four.$op_name(rhs),
                }
            }
        }

        impl $op<$name> for $scalar {
            type Output = $name;
            fn $op_name(self, rhs: $name) -> Self::Output {
                $name {
                    $axis_one: self.$op_name(rhs.$axis_one),
                    $axis_two: self.$op_name(rhs.$axis_two),
                    $axis_three: self.$op_name(rhs.$axis_three),
                    $axis_four: self.$op_name(rhs.$axis_four),
                }
            }
        }

        impl $op<$scalar> for &$name {
            type Output = $name;
            fn $op_name(self, rhs: $scalar) -> Self::Output {
                $name {
                    $axis_one: self.$axis_one.$op_name(rhs),
                    $axis_two: self.$axis_two.$op_name(rhs),
                    $axis_three: self.$axis_three.$op_name(rhs),
                    $axis_four: self.$axis_four.$op_name(rhs),
                }
            }
        }

        impl $op<&$name> for $scalar {
            type Output = $name;
            fn $op_name(self, rhs: &$name) -> Self::Output {
                $name {
                    $axis_one: self.$op_name(rhs.$axis_one),
                    $axis_two: self.$op_name(rhs.$axis_two),
                    $axis_three: self.$op_name(rhs.$axis_three),
                    $axis_four: self.$op_name(rhs.$axis_four),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! op_scalar_assign_impl {
    ($name:ident, $scalar:ident, $op:ident, $op_name:ident, $axis_one:ident, $axis_two:ident, $axis_three:ident) => {
        impl $op<$scalar> for $name {
            fn $op_name(&mut self, rhs: $scalar) {
                self.$axis_one.$op_name(rhs);
                self.$axis_two.$op_name(rhs);
                self.$axis_three.$op_name(rhs);
            }
        }
    };
    ($name:ident, $scalar:ident, $op:ident, $op_name:ident, $axis_one:ident, $axis_two:ident, $axis_three:ident, $axis_four:ident) => {
        impl $op<$scalar> for $name {
            fn $op_name(&mut self, rhs: $scalar) {
                self.$axis_one.$op_name(rhs);
                self.$axis_two.$op_name(rhs);
                self.$axis_three.$op_name(rhs);
                self.$axis_four.$op_name(rhs);
            }
        }
    };
}
