use super::generated;
use metrics::Unit;

impl From<generated::Unit> for Unit {
    fn from(value: generated::Unit) -> Self {
        match value {
            generated::Unit::Count => Unit::Count,
            generated::Unit::Percent => Unit::Percent,
            generated::Unit::Seconds => Unit::Seconds,
            generated::Unit::Milliseconds => Unit::Milliseconds,
            generated::Unit::Microseconds => Unit::Microseconds,
            generated::Unit::Nanoseconds => Unit::Nanoseconds,
            generated::Unit::Tebibytes => Unit::Tebibytes,
            generated::Unit::Gibibytes => Unit::Gibibytes,
            generated::Unit::Mebibytes => Unit::Mebibytes,
            generated::Unit::Kibibytes => Unit::Kibibytes,
            generated::Unit::Bytes => Unit::Bytes,
            generated::Unit::TerabitsPerSecond => Unit::TerabitsPerSecond,
            generated::Unit::GigabitsPerSecond => Unit::GigabitsPerSecond,
            generated::Unit::MegabitsPerSecond => Unit::MegabitsPerSecond,
            generated::Unit::KilobitsPerSecond => Unit::KilobitsPerSecond,
            generated::Unit::BitsPerSecond => Unit::BitsPerSecond,
            generated::Unit::CountPerSecond => Unit::CountPerSecond,
        }
    }
}

impl From<Unit> for generated::Unit {
    fn from(value: Unit) -> Self {
        match value {
            Unit::Count => generated::Unit::Count,
            Unit::Percent => generated::Unit::Percent,
            Unit::Seconds => generated::Unit::Seconds,
            Unit::Milliseconds => generated::Unit::Milliseconds,
            Unit::Microseconds => generated::Unit::Microseconds,
            Unit::Nanoseconds => generated::Unit::Nanoseconds,
            Unit::Tebibytes => generated::Unit::Tebibytes,
            Unit::Gibibytes => generated::Unit::Gibibytes,
            Unit::Mebibytes => generated::Unit::Mebibytes,
            Unit::Kibibytes => generated::Unit::Kibibytes,
            Unit::Bytes => generated::Unit::Bytes,
            Unit::TerabitsPerSecond => generated::Unit::TerabitsPerSecond,
            Unit::GigabitsPerSecond => generated::Unit::GigabitsPerSecond,
            Unit::MegabitsPerSecond => generated::Unit::MegabitsPerSecond,
            Unit::KilobitsPerSecond => generated::Unit::KilobitsPerSecond,
            Unit::BitsPerSecond => generated::Unit::BitsPerSecond,
            Unit::CountPerSecond => generated::Unit::CountPerSecond,
        }
    }
}
