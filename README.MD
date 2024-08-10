"

███████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████
███████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████
███████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████
███████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████
████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████  █████████████████████████
██████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████  ███████████████████████
███████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████        ████████████████████
██████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████     ████████████████████████
████████████████████████████████████████████████████████████████████████████████████████            ██████████████████████████      ███████████████████
████████████████████████████████████████████████████████████████████████████████████                 █████████████████████████████    █████████████████
██████████████████████████████████████████████████████████████████████████████████       █             ███████████████████████████     ████████████████
███████████████████████████████████████████████████████████████████████████████████  █       ██████   ████████████████████████████       ██████████████
████████████████████████████████████████████████████████████████████████████████████  ███ █     █  █ █████████████████████████████        █████████████
███████████████████████████████████████████████████████████████████████████████████████    █    █████████████ ███████ █████████████        █████████████
███████████████████████████████████████████████████████████████████████████████████      ██  █  █  █ ████████████                       ███ ███████████
████████████████████████████████████████████████████████████████████                       ███ █████████████████        █   █             █████████████
███████████████████████████████████████████████████████████████                               █         ██████████         █ ███       █████████████████
█████████████████████████████████████████        ███                  ████                                     ██        ███ █████████████████████
███████████████████████               ██████████         ████████████████████                                 ███         ████████████████████████ █
██████████████████     █████████████████████████████████████████████  █ ███████                             ████          █████████████████████     ██
███████████████████████   ██    █████████████████████████████████████████████     ██                    ████████  █        ███████               ███
█████████████████         ████████████████████████████████████████████████                         █   █████            ████       ████████████ █
    █              ████████████████████████████████████████████████████████████              █            ███  █  █         █████████████████████
      ███████  ███████████████████████████████████████████████████████████████████                ███████████   ██             ███████████████████ █████
█       ██████████████████████████████████████████████████████████████████████████  █  ██████████████████     █ ██             ███████████████  █  ████
███████████  █████████████████████████████████████████████████████████████                   ██         █  ███ ███      █████████ █████████  ████████
█████████████████████████████████████████████████              ██     ███                         █ █ █   ███ ███ ███████   ██ ████ ██████
████████████████████████████████████████      ██      █    █   █            ████ █       ██  ██████████████ ███████  █  ███████████  ███  █      █
  █ ████████████████████████████████            ████  █  █   █ ██          ██ █    █ ████   █           █          █ █████    ███████ ██       █ ██████
 █ ██ ███ █████          █             ███████████  █ █   █  ███         ████ █    ███          ██████    ████████████ ███ █     ███████    ███  ██████
████████ █    █    █                     ██████  ████     █   ███          █ █     ███      █                   █████ █ █    ██        █ ██          █
           ██                       ██████████████ ███    █ █ ████        ████          █   █     █    █       █         ██                       ██
                          ████████████████               █    █      █████                █               █ █████████             ████        ███████  "


" The shape of the body thus constructed was cubic,
  having six plane equilateral quadrangular bases.
  And seeing that there still remained one other compound figure,the fifth,
  God used it up for the Universe in his decoration thereof "
    -Plato, Timaeus 55c

This is my attempt at rewriting a doom style game engine in rust with as few dependencies as possible ~<3
im pretty new to the language so I am sure there will be lots to improve on

main.rs
    - entry point into the program, contains initialization of all of the neccesary sdl contexts
    - contains the global event loop that captures player input and calls the renderer each frame
    - the bottom of the main file contains the awfully written control flow to capture mouse input for the level editor

lib.rs
    - main library file; includes definitions of top-level game variables (such as the player and level)
      and their respective implimentations/associated functions
    - includes all imports of any dependencies and modules


renderer.rs
    - primary file for the 3D rendering engine
    - includes top level canvas definition and master draw order
    - includes most of the code to draw anything on the screen in perspective
    - note: does not include the rendering of the 2D level editor

grid.rs
    - primary file for the 2D level editor
    - includes defintion of the grid struct and associated functions
    - includes the file that generated level.rs