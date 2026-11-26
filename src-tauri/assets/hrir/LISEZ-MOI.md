# Réponses de tête mesurées (mode immersif, « Tête mesurée »)

Source : base **SADIE II**, AudioLab, Department of Electronic Engineering, University of
York (version 2-2, Zenodo, doi:10.5281/zenodo.12092466). Sujet **D1** (tête de mesure
Neumann KU100), fichiers `D1_44K_16bit_256tap_FIR_SOFA.sofa` et
`D1_48K_24bit_256tap_FIR_SOFA.sofa`. Article de référence : doi:10.3390/app8112029.

Licence : Apache 2.0 (`LICENSE-SADIE-II.txt`), « Copyright 2018, University of York ».

Modifications : seul le plan horizontal (élévation 0°) est gardé, tous les 5° (72 directions,
sens trigonométrique : 0° devant, 90° à gauche), 2 oreilles × 256 coefficients, normalisé
pour une énergie moyenne de 1 par oreille et par direction, enregistré en flottants 32 bits
petit-boutistes `[direction][oreille][coefficient]` (oreille 0 = gauche).
