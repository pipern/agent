# Changelog

## 0.1.0 - 2021-05-17

### Added
- Apache license v2.0 set ([#23]).
- Krustlet based agent implementation created ([#1], [#18], [#26], [#35], [#40]).
- Functionality to stop and restart processes added ([#25]).
- Agent restart without impacting running services enabled ([#63]).
- Rendering of template variables to environment variables added ([#30]).
- Setting of pod condition "ready" for state "running" added ([#32]).
- Support for command line parameters added ([#36], [#50], [#72], [#109]).
- Integration with systemd implemented ([#43], [#53], [#100], [#152]).
- Dependabot and security audit enabled ([#56], [#57]).
- Building and publishing of nightly deb and rpm packages added ([#73], [#78], [#94], [#110], [#144]).
- Bootstrapping of certificates and kubeconfig added ([#77]).
- Support for running of services as application users added ([#79]).
- Retrieval of container logs with kubectl logs implemented ([#135]).
- Configuration of terminationGracePeriodSeconds considered in systemd units ([#138]).
- Systemd dependency adapted so that it is compatible with systemd version 241 ([#145]).