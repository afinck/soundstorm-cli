%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: soundstorm-cli
Summary: Plays Soundstorm Internet-Radio Stream


Version: @@VERSION@@
Release: @@RELEASE@@%{?dist}
License: MIT or ASL 2.0
Group: Applications/System
Requires: mpv
Source0: %{name}-%{version}.tar.gz

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

%description
For me this is the only radio I need.
Much better than what we used to have around here.
No distraction - no noise - just music.
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}

%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*
