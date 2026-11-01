use eframe::Storage;
use eframe::egui::{Key, vec2};
use qix_core::tuning::{
    CLAIM_THRESHOLD_DEFAULT, CLAIM_THRESHOLD_RANGE, STARTING_LIVES, STARTING_LIVES_RANGE,
};
use qix_core::{ClaimThreshold, Rules, StartingLives};

use crate::bindings::{Control, KeyBindings};
use crate::persist::{SETTINGS_KEY, load_settings, save_settings};
use crate::persist_test::MemoryStorage;
use crate::settings::{BASE_WINDOW_SIZE, SCALE_DEFAULT, SCALE_RANGE, Scale, Settings};
use crate::{DEFAULT_WINDOW_SIZE, MIN_WINDOW_SIZE};

/// The stored value of the default settings.
const DEFAULTS: &str = "(threshold:0.75,starting_lives:3,keys:(up:ArrowUp,down:ArrowDown,\
     left:ArrowLeft,right:ArrowRight,fast:Z,slow:X),scale:2.0)";

/// A storage holding `value` under [`SETTINGS_KEY`].
fn stored(value: &str) -> MemoryStorage {
    let mut storage = MemoryStorage::default();
    storage.set_string(SETTINGS_KEY, value.to_owned());
    storage
}

/// Settings with every field off its default.
fn edited() -> Settings {
    let mut keys = KeyBindings::default();
    assert!(keys.bind(Control::Up, Key::W));
    assert!(keys.bind(Control::Slow, Key::Space));
    let settings = Settings {
        threshold: ClaimThreshold::new(*CLAIM_THRESHOLD_RANGE.end()),
        starting_lives: StartingLives::new(*STARTING_LIVES_RANGE.start()),
        keys,
        scale: Scale::new(*SCALE_RANGE.end()),
    };
    let defaults = Settings::default();
    assert_ne!(settings.threshold, defaults.threshold);
    assert_ne!(settings.starting_lives, defaults.starting_lives);
    assert_ne!(settings.keys, defaults.keys);
    assert_ne!(settings.scale, defaults.scale);
    settings
}

#[test]
fn the_defaults_come_from_the_tuning() {
    let settings = Settings::default();

    assert_eq!(settings.threshold.fraction(), CLAIM_THRESHOLD_DEFAULT);
    assert_eq!(settings.starting_lives.get(), STARTING_LIVES);
    assert_eq!(settings.keys, KeyBindings::default());
    assert_eq!(settings.scale.get(), SCALE_DEFAULT);
    assert_eq!(settings.rules(), Rules::default());
}

#[test]
fn the_rules_are_the_threshold_and_the_starting_lives() {
    let settings = edited();

    let rules = settings.rules();

    assert_eq!(rules.threshold, settings.threshold);
    assert_eq!(rules.starting_lives, settings.starting_lives);
}

#[test]
fn a_scale_in_the_range_is_kept() {
    for factor in [*SCALE_RANGE.start(), SCALE_DEFAULT, *SCALE_RANGE.end()] {
        assert_eq!(Scale::new(factor).get(), factor);
    }
}

#[test]
fn a_scale_outside_the_range_is_clamped_into_it() {
    let (low, high) = (*SCALE_RANGE.start(), *SCALE_RANGE.end());

    assert_eq!(Scale::new(0.0).get(), low);
    assert_eq!(Scale::new(-1.0).get(), low);
    assert_eq!(Scale::new(f32::NEG_INFINITY).get(), low);
    assert_eq!(Scale::new(high + 1.0).get(), high);
    assert_eq!(Scale::new(f32::INFINITY).get(), high);
}

#[test]
fn a_nan_scale_is_the_default() {
    assert_eq!(Scale::new(f32::NAN), Scale::default());
}

#[test]
fn the_window_size_is_the_base_size_times_the_scale() {
    let base = vec2(BASE_WINDOW_SIZE[0], BASE_WINDOW_SIZE[1]);

    assert_eq!(
        Scale::new(*SCALE_RANGE.end()).window_size(),
        base * *SCALE_RANGE.end()
    );
    assert_eq!(Scale::default().window_size(), DEFAULT_WINDOW_SIZE.into());
}

#[test]
fn the_smallest_scale_fits_the_smallest_window() {
    let size = Scale::new(*SCALE_RANGE.start()).window_size();

    assert!(
        size.x >= MIN_WINDOW_SIZE[0] && size.y >= MIN_WINDOW_SIZE[1],
        "{size:?}"
    );
}

#[test]
fn saved_settings_load_back_the_same() {
    for settings in [Settings::default(), edited()] {
        let mut storage = MemoryStorage::default();

        save_settings(&mut storage, &settings);

        assert_eq!(load_settings(Some(&storage)), settings);
    }
}

#[test]
fn the_settings_are_stored_as_readable_fields() {
    let mut storage = MemoryStorage::default();

    save_settings(&mut storage, &Settings::default());

    assert_eq!(storage.get_string(SETTINGS_KEY).as_deref(), Some(DEFAULTS));
}

#[test]
fn without_storage_or_stored_settings_the_defaults_load() {
    assert_eq!(load_settings(None), Settings::default());
    assert_eq!(
        load_settings(Some(&MemoryStorage::default())),
        Settings::default()
    );
}

#[test]
fn settings_that_do_not_decode_load_as_the_defaults() {
    for value in [
        "",
        "not ron",
        "(threshold:\"high\")",
        "(keys:(up:NoSuchKey))",
    ] {
        assert_eq!(
            load_settings(Some(&stored(value))),
            Settings::default(),
            "{value:?}"
        );
    }
}

#[test]
fn stored_values_out_of_range_are_clamped() {
    let storage = stored("(threshold:0.1,starting_lives:99,scale:10.0)");

    let settings = load_settings(Some(&storage));

    assert_eq!(
        settings.threshold.fraction(),
        *CLAIM_THRESHOLD_RANGE.start()
    );
    assert_eq!(settings.starting_lives.get(), *STARTING_LIVES_RANGE.end());
    assert_eq!(settings.scale.get(), *SCALE_RANGE.end());
}

#[test]
fn stored_values_below_the_range_are_clamped() {
    let storage = stored("(threshold:1.5,starting_lives:0,scale:0.1)");

    let settings = load_settings(Some(&storage));

    assert_eq!(settings.threshold.fraction(), *CLAIM_THRESHOLD_RANGE.end());
    assert_eq!(settings.starting_lives.get(), *STARTING_LIVES_RANGE.start());
    assert_eq!(settings.scale.get(), *SCALE_RANGE.start());
}

#[test]
fn missing_fields_load_as_their_defaults() {
    let settings = load_settings(Some(&stored("(starting_lives:5)")));

    assert_eq!(
        settings,
        Settings {
            starting_lives: StartingLives::new(5),
            ..Settings::default()
        }
    );
}

#[test]
fn broken_key_bindings_load_as_the_default_ones_and_keep_the_rest() {
    for keys in [
        "(up:W,down:W,left:A,right:D,fast:J,slow:K)",
        "(up:Enter,down:S,left:A,right:D,fast:J,slow:K)",
        "(up:W,down:S,left:A,right:D,fast:F1,slow:K)",
    ] {
        let storage = stored(&format!("(starting_lives:5,keys:{keys})"));

        let settings = load_settings(Some(&storage));

        assert_eq!(settings.keys, KeyBindings::default(), "{keys}");
        assert_eq!(settings.starting_lives.get(), 5, "{keys}");
    }
}

#[test]
fn valid_stored_key_bindings_load() {
    let storage = stored("(keys:(up:W,down:S,left:A,right:D,fast:J,slow:K))");

    let keys = load_settings(Some(&storage)).keys;

    let expected = [Key::W, Key::S, Key::A, Key::D, Key::J, Key::K];
    for (control, key) in Control::ALL.into_iter().zip(expected) {
        assert_eq!(keys.key(control), key, "{control:?}");
    }
}
