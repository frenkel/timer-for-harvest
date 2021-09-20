%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: timer-for-harvest
Summary: Timer for Harvest
Version: @@VERSION@@
Release: @@RELEASE@@
License: BSD-2-Clause
Group: Applications/System
Source0: %{name}-%{version}.tar.gz
URL: https://github.com/frenkel/timer-for-harvest

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

%description
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
/usr/share/applications/timer-for-harvest.desktop
