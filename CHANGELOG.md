# Changelog

## [UNRELEASED] - YYYY-MM-DD

- Re-enable the ability to resize the main window, implemented by @mlunax.
- The license of Timer for Harvest will no longer show up as proprietary in GNOME Software.

## [0.3.7] - 2021-08-26

- Show correct button label when adding hours instead of starting a new timer, implemented by @mlunax.
- Fixed a crash on resume-from-suspend because the time entries could not load while network was not up yet.

## [0.3.6] - 2021-03-12

This release includes escaping for special characters such as & in task, project en client names. Thanks @wvengen for reporting this bug.
Furthermore the update link is now clickable thanks to @iatanas0v.

## [0.3.5] - 2021-02-26

Thanks to @KillerCodeMonkey we have another release! This time he contributed changes to the main window that keep it a static size with a scrollbar. Thanks again @KillerCodeMonkey!

## [0.3.4] - 2021-02-05

Thanks to @KillerCodeMonkey we now have multi-line description support!
We also had to switch to a different client\_id for the authorization at Harvest. You shouldn't notice this but we're reporting this so that you know it is not a security incident.

## [0.3.3] - 2020-10-23

This release includes the following new features and bugfixes:

- Auto refresh when receiving window focus, idea by @Flimm
- Add client name to project chooser, requested by @jarnalyrkar
- New entry's are now added on the chosen date instead of the current date
- Edit button is now re-enabled when cancelling the edit popup


## [0.3.2] - 2020-08-21

This release consist of a major architectural rewrite. As an end user this shouldn't be noticeable, except for some interface alignment changes. However one user noticeable feature was added: the application will now check whether a new version is available and show a notification when one is available. This works by checking a DNS TXT record, so we cannot misuse this to track usage of the application.

## [0.3.1] - 2020-06-26

This release adds two new features contributed by @jaspervandenberg:

- Add a button to go to the current date
- Ask for confirmation before deleting an entry

## [0.3.0] - 2020-06-09

This release includes various interface improvements and one big new feature: the ability to switch to different days.

## [0.2.1] - 2020-04-14

This is a very small release that fixes one important bug: the project code can be null, which would result in a crash.

## [0.2.0] - 2020-04-08

This release improves the user experience a lot. All API communication has been moved to a background thread, resulting in a more response user interface. Furthermore, a number of error messages have been approved, which should make debugging problems easier.

## [0.1.12] - 2020-02-19

The main window acquired a lot of white spacing after enabling truncate on the labels. This release reduces it drastically.

## [0.1.11] - 2020-02-18

This release mostly documents the application, but it also fixes a line wrapping problem for long project names

## [0.1.10] - 2020-02-11

This release loads project data on startup instead of everytime you click a button. This makes using it on a daily basis a lot faster, at the expense of a slower startup time.

## [0.1.9] - 2020-01-14

This release is the first release with packages for Fedora 31 and Ubuntu 19.10. A small bug that would occur on Ubuntu when authorizing was fixed in this release as well.

## [0.1.8] - 2019-12-31

This release adds the ability to save and load the previously obtained OAuth tokens to disk. This means that the authorization flow needs to be run only once every two weeks, making it a lot more user-friendly.

## [0.1.7] - 2019-12-29

This is the first user-friendly release because it replaces the usage of developer tokens with an OAuth authorization flow. Currently the obtained authorization is not remembered between different application launches, but support for that will follow in a later release.

## [0.1.6] - 2019-12-27

Improvements in this release:

- Re-order time entries of the day correctly, newest at the bottom
- Don't require a terminal when running, by removing println!() statements

## [0.1.5] - 2019-12-26

This release includes some important refactoring, which allowed us to add the ability for running timers to update their labels.

## [0.1.4] - 2019-12-24

This release makes usage a lot faster by adding the following features:

- Auto completion in the project chooser combobox
- Auto completion in the task chooser combobox
- Close timer popup without saving by pressing escape
- Open timer popup by pressing n

## [0.1.3] - 2019-12-10

This release adds some more polish to the whole application. Notable changes include:

- Wrap notes in main window.
- Don't crash when time entry has no notes.
- Pressing enter in popup now defaults to saving the timer.
- Task loading has been modified to don't rely on admin-permission API calls. This change also allowed for an extra API call to be removed, resulting in a nicer workflow.

## [0.1.2] - 2019-12-03

This release adds some small but important improvements.

- Implement workaround for updating running timers without overwriting hours
- Show more time entry info, add some style improvements
- Order projects case-insensitive
- Don't crash after switching project where task is missing

## [0.1.1] - 2019-11-28

This release fixes various bugs:

- Fix bug in time formatting where 1 minute would show as 10
- Don't crash when editing timers without notes
- Refresh time entries list after edits

## [0.1.0] - 2019-11-26

This is the first public release. Basic functionality works, including:

- Show entry's of today
- Start new timer or
- Restart earlier timer
- Stop running timer
- Add new entry with duration
- Refresh time entries list on F5 press
