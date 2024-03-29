global_conf = configuration_data()
global_conf.set_quoted('APP_ID', application_id)
global_conf.set_quoted('PKGDATADIR', pkgdatadir)
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('VERSION', version + version_suffix)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('LOCALEDIR', localedir)
config = configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: global_conf
)
# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.project_build_root() / 'src' / 'config.rs',
  meson.project_source_root() / 'src' / 'config.rs',
  check: true
)

sources = files(
  'backend/card.rs',
  'backend/data.rs',
  'backend/image.rs',
  'backend/log.rs',
  'backend/mod.rs',
  'backend/scraper.rs',
  'backend/social.rs',
  'models/log_item.rs',
  'models/log_model.rs',
  'models/metadata_item.rs',
  'models/mod.rs',
  'widgets/card.rs',
  'widgets/image.rs',
  'widgets/log_dialog.rs',
  'widgets/data_dialog.rs',
  'application.rs',
  'config.rs',
  'i18n.rs',
  'main.rs',
  'window.rs',
)

custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  depends: resources,
  command: [
    cargo_script,
    meson.project_build_root(),
    meson.project_source_root(),
    '@OUTPUT@',
    profile,
    meson.project_name(),
  ]
)
