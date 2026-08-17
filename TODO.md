# TODO

- repo
    - Thorough readme with goals/plans/pictures
- build system
    - unify caches and downloads between rust invoked by editor, ksb, and buildroot
        - Maybe vendor deps for everything outside of buildroot as well
- firmware
    - increase timer accuracy
    - enable watchdog timer
- operating system
    - decrease boot time
    - Rip out cardinal, replace with open source vst instruments
    - keyboard-daemon
        - replace libc with existing safe wrappers
        - revisit velocity calculations
        - screen
            - Some animation when inputs happen
            - Ability to switch onboard instruments
            - show screensaver when UI hasn't updated in a while (OLED burn in prevention)