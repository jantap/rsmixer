Actions have on of two formats:

```
mute
raise_volume(5)
```

Most actions are just words seperated by underscores, however some take a parameter in form of a number.

| name                 | description                                             | argument                     |
| -------------------- | ------------------------------------------------------- | ---------------------------- |
| up(arg)              | select an option higher than the currently selected one | number of places to move     |
| down(arg)            | select an option lower than the currently selected one  | number of places to move     |
| lower_volume(arg)    | lower the volume of the currently selected entry        | how much to lower the volume |
| raise_volume(arg)    | raise the volume of the currently selected entry        | how much to raise the volume |
| mute                 | mute the currently selected entry                       |                              |
| hide                 | hide sink inputs/source outputs of current sink/source  |                              |
| show_output          | show output tab                                         |                              |
| show_input           | show input tab                                          |                              |
| show_cards           | show cards tab                                          |                              |
| cycle_pages_forward  | cycle to the next tab                                   |                              |
| cycle_pages_backward | cycle to the previous tab                               |                              |
| context_menu         | open context menu of the currently selected entry       |                              |
| close_context_menu   | close the currently open context menu                   |                              |
| confirm              | confirm selection in currently open context menu        |                              |
| help                 | show help screen                                        |                              |
| exit                 | close rsmixer                                           |                              |
