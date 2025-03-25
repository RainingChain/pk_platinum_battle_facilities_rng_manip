# Pokemon Platinum Battle Tower RNG Manipulation

## Main use case
 - Find the number of RNG advances to perform to battle against of series of easy Pokemons in Pokemon Platinum Battle Tower
 - [Interactive tool](https://scripterswar.com/pokemon/BattleFacilities/Platinum/RngManipulation)
 - [Technical details](https://scripterswar.com/pokemon/articles/Platinum_Battle_Tower_RNG_Manipulation)

## Sub-use cases
- Determine your seed based on first seen trainer Pokemons
- Determine advances for easiest trainer teams
- Determine all 7 trainer teams based on first seen trainer Pokemons
- Determine all 7 trainer teams based on same day seed

## Determine RNG manips for easy Pokemons
- Step 1: Determine your seed based on first seen trainer Pokemons
  - Important: Save the game, then advance the DS clock by 1 day.
  - If your goal is to manipulate RNG for Single, boot the game and start/continue your Double streak.
    - The tool doesn't support yet RNG manipulation for Double.
  - Write down the trainer names and their Pokemons. At least 2 trainers and their Pokemon are recommended.
  - Execute `pk_platinum_battle_facilities_rng_manip.exe find_seeds --facility <> --wins <> --battled_trainers <> --battled_pokemons <>`
    - Ex: `pk_platinum_battle_facilities_rng_manip.exe find_seeds --facility double --wins 0 --battled_trainers Vincent,Tia --battled_pokemons Baltoy,Vulpix,Gible,Cyndaquil,Castform,Remoraid,Mudkip,Lombre`
    - returns `--same_day_seed 0xb6d64383 --diff_day_seed 0x9bceb5e5`


- Step 2: Find RNG manips for easy Pokemons using your --same_day_seed and --diff_day_seed
  - Run `pk_platinum_battle_facilities_rng_manip.exe search_easy --facility <> --wins <> --same_day_seed <> --diff_day_seed <>`
    - With the streak info you want to continue.
  - Ex: `pk_platinum_battle_facilities_rng_manip.exe search_easy --facility single --wins 49 --same_day_seed 0x7b3dd48e --diff_day_seed 0x80720009`
  - returns
    ```
    Rating: 18.0 with Infernape, Jolly, Choice Band, Close Combat,Flare Blitz,Earthquake,Thunder Punch (#3)
      Different day RNG advance: 178256 days, 2000-01-01 -> 2099-12-31 (x4), ,2000-01-01 -> 2088-01-19
        Different day seed after advances: 0xb527a779
      Same day RNG advance (start and abandon streak): 9
      Same day Seed (after day update): 0xfb15520f, Team Seed: 0x3a3f716c
    Socialite Janice (Rating: 3, Player Choice Move: Close Combat)
      Absol 3 w/ Pressure (Rating: 1)
      Ambipom 3 w/ Pickup (Rating: 1)
      Glalie 3 w/ Inner Focus (Rating: 1)
    Beauty Nadia (Rating: 2, Player Choice Move: Close Combat)
      Omastar 2 w/ Swift Swim (Rating: 1)
      Cacturne 2 w/ Sand Veil (Rating: 1)
      Dusclops 2 w/ Pressure (Rating: 0)
    Idol Nissa (Rating: 3, Player Choice Move: Close Combat)
      Flareon 2 w/ Flash Fire (Rating: 1)
      Glaceon 4 w/ Snow Cloak (Rating: 1)
      Umbreon 2 w/ Synchronize (Rating: 1)
    PI Chester (Rating: 2, Player Choice Move: Close Combat)
      Dewgong 3 w/ Thick Fat (Rating: 1)
      Glalie 2 w/ Ice Body (Rating: 1)
      Weezing 3 w/ Levitate (Rating: 0)
    Cameraman Darren (Rating: 3, Player Choice Move: Flare Blitz)
      Alakazam 4 w/ Synchronize (Rating: 1)
      Bronzong 1 w/ Levitate (Rating: 1)
      Magnezone 3 w/ Magnet Pull (Rating: 1)
    Pokéfan♀ Pandora (Rating: 2, Player Choice Move: Flare Blitz)
      Breloom 4 w/ Effect Spore (Rating: 1)
      Slowking 4 w/ Own Tempo (Rating: 0)
      Absol 4 w/ Super Luck (Rating: 1)
    Roughneck Ross (Rating: 3, Player Choice Move: Flare Blitz)
      Honchkrow 3 w/ Insomnia (Rating: 1)
      Absol 1 w/ Super Luck (Rating: 1)
      Mismagius 3 w/ Levitate (Rating: 1)
    ```
  - Additional optional arguments:
    - `--max_diff_day_change`: Maximum number of DS clock date change. Increasing this value returns better results, but will take for time to setup the RNG manipulation. Default: 10
    - `--max_same_day_adv`: Maximum number of same day advance (starting then abandoning a streak). Increasing this value returns better results, but will take for time to setup the RNG manipulation. Default: 10
    - `--max_threads`: Maximum number of threads for finding seeds and searching easy teams. A higher value will execute faster but may use more CPU ressources. Default: PC's core count.

## Lead-only Guaranteed Victory
  - In Singles, team seed 0xbf630d93 results in Regice 4,Articuno 4,Lapras 1,Porygon2 4,Lucario 4,Glaceon 4,Porygon-Z 4,Probopass 4,Gardevoir 4,Victreebel 3,Vileplume 3,Miltank 3,Froslass 3,Shiftry 3,Sceptile 3,Porygon-Z 4,Houndoom 4,Porygon2 4,Articuno 3,Mamoswine 4,Abomasnow 4.
    - 100% guaranteed victory with Infernape@Life Orb, Jolly, 31 IV Attack, 31 IV Speed, 252 EV Attack, 252 EV Speed, Close Combat, Flare Blitz

## For developpers
- Compile exe:
  - cargo run

- Compile wasm:
  - wasm-pack build --target web

## Credits
  - RainingChain
  - [Platinum decompil project](https://github.com/pret/pokeplatinum)