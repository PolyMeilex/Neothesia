# Plan d'Intégration ROLI LUMI Keys & État d'Avancement

## 📝 Objectif
Transformer Neothesia en une plateforme d'apprentissage interactive en exploitant le matériel ROLI LUMI Keys (LEDs par touche, mode "Wait", MPE).

---

## ✅ Ce qui a été fait (Accomplished)

### Phase 1 : Communication SysEx & Protocole
- [x] **Protocole SysEx ROLI :** Implémentation du packing de données 7-bits (BitArray) conforme aux spécifications ROLI.
- [x] **Contrôleur LUMI (`lumi_controller.rs`) :** Création d'un module gérant l'allumage des LEDs RGB, la luminosité et les modes de couleur.
- [x] **Tests Unitaires :** 9 tests unitaires vérifiant que les séquences SysEx générées correspondent exactement aux payloads attendus (Rainbow, Piano, Night, Luminosité 0-100%).

### Phase 2 : Architecture & Connectivité
- [x] **Sortie MIDI Dédiée :** Neothesia ouvre désormais un port de sortie MIDI "miroir" dédié au matériel LUMI dès que le clavier est choisi en entrée. Cela permet d'envoyer des commandes SysEx sans interférer avec la synthèse sonore.
- [x] **Interface Paramètres (Menu) :** Ajout d'une section **LUMI Hardware** permettant de changer la luminosité et le mode d'éclairage en temps réel depuis le menu principal.

### Phase 3 : Moteur de Jeu
- [x] **Feedback Visuel (Playing Scene) :**
    - [x] Allumage des touches correspondant aux notes qui tombent.
    - [x] **Hinting :** Éclairage tamisé des touches 2 secondes avant l'arrivée de la note pour guider l'utilisateur.
- [x] **Mode Pause (Wait Mode) :** Intégration du système "PlayAlong" pour stopper le défilement tant que les touches attendues ne sont pas pressées.

---

## 🛠 Ce qui reste à faire (Remaining)

### Améliorations de l'Interface (Settings & UI)
- [ ] **Visibilité Dynamique :** Masquer la section "LUMI Hardware" si aucun clavier LUMI n'est détecté/sélectionné.
- [ ] **Boutons Répétitifs :** Permettre l'incrémentation continue de la luminosité en maintenant les boutons `+`/`-` enfoncés.
- [ ] **Écran de Transition :** Ajouter un écran intermédiaire après "Play" pour choisir le mode (Watch / Learn / Play) et sélectionner les mains (gauche/droite/les deux).

### Gameplay & MIDI
- [ ] **Mode "Human" :** Assurer que le son se déclenche immédiatement à l'appui de la touche en mode "Wait", et non seulement à la fin de la plage de la note.
- [ ] **Masquage de Canaux :** Permettre de cacher certains canaux MIDI de la visualisation cascade.
- [ ] **Support MPE (Phase 4) :** Utiliser la pression (Aftertouch) et le Pitch Bend pour moduler la couleur ou l'intensité des LEDs.

### Finitions
- [ ] **Écran de Score :** Afficher un résumé de la performance à la fin des morceaux (précision, timing).
- [ ] **Gestion des Soundfonts :** Faciliter le changement de dossiers Soundfont et le cycle entre eux en cours de jeu.

---

## 🛠 Diagnostic Technique Récent
La correction majeure a été la séparation des flux MIDI. Précédemment, les commandes SysEx étaient envoyées au synthétiseur audio au lieu du clavier physique. L'ouverture automatique du port de sortie LUMI par Neothesia a résolu les problèmes de communication matérielle.
