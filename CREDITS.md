# Slice: 4D Shooter - Attribution and Licensing

This document outlines the copyright and licensing information for the various components of the Slice: 4D Shooter project.
## 1. Source Code

The entire source code of the **Slice: 4D Shooter** project is licensed under the **MIT License**.

MIT License

Copyright (c) 2026 Molokov Timofei

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.

A full copy of the license is available in the project's root directory in the file LICENSE.txt.

## 2. Third-Party Libraries

This project includes third-party libraries and code used under their respective licenses.

### 2.1. WinSparkle

The project uses the **WinSparkle** library for automatic updates on Windows.

*   **Location:** `dll/` directory
*   **Copyright:** (c) 2009-2025 Vaclav Slavik
*   **License:**
    ```
    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal in
    the Software without restriction, including without limitation the rights to
    use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
    of the Software, and to permit persons to whom the Software is furnished to do
    so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.
    ```
*   **Note:** This product includes software developed by the OpenSSL Project for use in the OpenSSL Toolkit (http://www.openssl.org/).

### 2.2. Matchbox Signaling

The project includes a modified version of the `matchbox_signaling` Rust crate.

*   **Location:** `matchbox_signaling_modified/` directory
*   **Original License:** Dual-licensed under either the **MIT License** or the **Apache License 2.0** (at the user's choice).
*   **Modifications:** The source code has been modified. The original licenses, README, and a `NOTICE` file describing all modifications are preserved within the directory.
*   **Full original copyright notices and license texts are available in the aforementioned directory.**

## 3. Assets (Audio, Textures, Graphics)
### 3.1. Third-Party Audio

The sound effects listed below are used under their respective Creative Commons licenses. Attribution to the original authors is required as specified by their licenses.

| File Path | Author | Source | License | Modifications |
| :--- | :--- | :--- | :--- | :--- |
| `src/assets/sounds/shifting_along_w.wav` | MrJmix | [Link](https://freesound.org/people/MrJmix/sounds/331078/) | CC BY 1.0 | Trimmed, fade out applied. |
| `src/assets/sounds/player_died.wav` | PJ_Bear, thehorriblejoke | [Link](https://freesound.org/people/PJ_Bear/sounds/708967/), [Link](https://freesound.org/people/thehorriblejoke/sounds/505751/) | CC0 1.0, CC0 1.0 | Trimmed, fade out applied, combined. |
| `src/assets/sounds/player_hit_signal.wav` | Sojan | [Link](https://freesound.org/people/Sojan/sounds/334298/) | CC0 1.0 | Combined. |
| `src/assets/sounds/rotating_around_w.wav` | vumseplutten1709 | [Link](https://freesound.org/people/vumseplutten1709/sounds/237269/) | CC BY 3.0 | Trimmed, fade out applied. |
| `src/assets/sounds/player_explosion.wav` | psychedelic_hands | [Link](https://freesound.org/people/psychedelic_hands/sounds/144469/) | CC BY 3.0 | Trimmed, fade out applied, combined. |
| `src/assets/sounds/player_death_signal.wav` | newagesoup | [Link](https://freesound.org/people/newagesoup/sounds/347310/) | CC BY 4.0 | Trimmed, fade out applied. |
| `src/assets/sounds/get_score.wav` | newagesoup | [Link](https://freesound.org/people/newagesoup/sounds/347310/) | CC BY 4.0 | — |
| `src/assets/sounds/team_win.wav` | SilverIllusionist | [Link](https://freesound.org/people/SilverIllusionist/sounds/665205/) | CC BY 4.0 | Trimmed, fade out applied. |
| `src/assets/sounds/team_lost.wav` | smokinghotdog | [Link](https://freesound.org/people/smokinghotdog/sounds/584246/) | CC0 1.0 | Trimmed, fade out applied. |
| `src/assets/sounds/lost_score.wav` | jolup123 | [Link](https://freesound.org/people/jolup123/sounds/668791/) | CC0 1.0 | — |
| `src/assets/sounds/flag_returned.wav` | jtombas | [Link](https://freesound.org/people/jtombas/sounds/722736/) | CC0 1.0 | — |
| `src/assets/sounds/flag_captured.wav` | Win Spacey | [Link](https://freesound.org/people/GameAudio/sounds/220184/) | CC0 1.0 | Trimmed, pitch changed, combined. |
| `src/assets/sounds/holegun_charging.wav` | Tim_Verberne | [Link](https://freesound.org/people/Tim_Verberne/sounds/514082/) | CC0 1.0 | Trimmed, fade out applied. |
| `src/assets/sounds/player_respawned.wav` | colorsCrimsonTears | [Link](https://freesound.org/people/colorsCrimsonTears/sounds/562292/) | CC0 1.0 | — |
| `src/assets/sounds/player_explosion.wav` | Kodack | [Link](https://freesound.org/people/Kodack/sounds/258194/) | CC0 1.0 | — |
| `src/assets/sounds/holegun_shot.wav` | Emanuele_Correani | [Link](https://freesound.org/people/Emanuele_Correani/sounds/260155/) | CC BY 4.0 | Trimmed, pitch changed, combined. |
| `src/assets/sounds/player_hited.wav` | Mrthenoronha | [Link](https://freesound.org/people/Mrthenoronha/sounds/507704/) | CC BY 4.0 | Trimmed, pitch changed, combined. |
| `src/assets/sounds/machinegun_shot.wav` | Astounded | [Link](https://freesound.org/people/Astounded/sounds/479353/) | CC0 1.0 | — |
| `src/assets/sounds/shotgun_shot.wav` | Artninja, Emanuele Correani | [Link](https://freesound.org/people/Artninja/sounds/784935/), [Link](https://freesound.org/people/Emanuele_Correani/sounds/260155/) | CC BY 4.0, CC BY 4.0 | Mixed, trimmed, fade out applied. |
| `src/assets/sounds/shotgun_shot_impact.wav` | Metzik | [Link](https://freesound.org/people/Metzik/sounds/459782/) | CC BY 4.0 | Trimmed, fade out applied. |
| `src/assets/sounds/new_projection.wav` | unfa | [Link](https://freesound.org/people/unfa/sounds/244266/) | CC0 1.0 | Trimmed. |
| `src/assets/sounds/player_get_scanned.wav` | unfa | [Link](https://freesound.org/people/unfa/sounds/244266/) | CC0 1.0 | Trimmed. |
| `src/assets/sounds/switch_weapon.wav` | moodyfingers, suonho | [Link](https://freesound.org/people/moodyfingers/sounds/730213/), [Link](https://freesound.org/people/suonho/sounds/4014/) | CC BY 4.0, CC BY 4.0 | Mixed, trimmed. |
| `src/assets/sounds/charging_w_jump.wav` | KVV_Audio, Tim_Verberne | [Link](https://freesound.org/people/KVV_Audio/sounds/741272/), [Link](https://freesound.org/people/Tim_Verberne/sounds/514082/) | CC BY 4.0, CC0 1.0 | Mixed, trimmed, reverb applied, amplified. |
| `src/assets/sounds/w_jump.wav` | Funky_Audio | [Link](https://freesound.org/people/Funky_Audio/sounds/729389/) | CC0 1.0 | Trimmed, reverb applied, amplified, speed changed. |
| `src/assets/sounds/projection_captured.wav` | anthonychartier2020 | [Link](https://freesound.org/people/anthonychartier2020/sounds/560188/) | CC BY 4.0 | Speed changed, pitch shifted, fade in/out applied. |
| `src/assets/sounds/player_respawned.wav` | colorsCrimsonTears | [Link](https://freesound.org/people/colorsCrimsonTears/sounds/562292/) | CC0 1.0 | — |

    CC0 1.0 Universal is a public domain dedication. You can copy, modify, and distribute the sound, even for commercial purposes, without asking permission. See the license deed.

    CC BY 3.0/4.0 requires attribution to the creator. You must give appropriate credit, provide a link to the license, and indicate if changes were made. See the CC BY 4.0 deed or CC BY 3.0 deed.

### 3.2. Original Project Assets


All other assets in this project — including but not limited to textures, graphics, models, shaders, UI elements, and original sound effects not listed above — are original works created by **Molokov Timofei**.

    These assets are released under the **Creative Commons Zero v1.0 Universal (CC0 1.0)** license.

    CC0 1.0 Universal

    The author has dedicated this work to the public domain by waiving all
    copyright and related rights to the extent allowed by law.

    Full legal text:  
    https://creativecommons.org/publicdomain/zero/1.0/legalcode

Exclusion: All files located in the ./media/ directory are the exclusive property of Molokov Timofei and are All Rights Reserved. These files are included for demonstration purposes only within this project and are not licensed for redistribution or reuse under the MIT or CC0 1.0 licenses.

## 4. Trademark and Project Name

The name "Slice: 4D Shooter", associated logos, and any other related trademarks are the exclusive intellectual property of Molokov Timofei.

These trademarks are not licensed under the MIT License or CC0 1.0. Use of the project name or trademarks is only permitted to identify the original source of the software and assets. They may not be used to endorse or promote derivative works without explicit, prior written permission from the copyright holder.

## Summary of Licensing

| Component | License | Copyright Holder |
| --- | --- | --- |
| Source Code | MIT License | Molokov Timofei |
| Original Assets | CC0 1.0 (Public Domain) | Molokov Timofei |
| `./media/` directory assets | All Rights Reserved | Molokov Timofei |
| Third-Party Sounds | CC0 / CC BY 3.0 / CC BY 4.0 | Various |
| Project Name & Logo | Trademark (All Rights Reserved) | Molokov Timofei |