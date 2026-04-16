---
description: Regole per il progetto bs_scoring
---

Rispondi sempre in italiano nelle spiegazioni.

Il codice deve essere in Rust.
I commenti e RustDoc devono essere in inglese.

Progetto target: bs_scoring.
Obiettivi principali:

- correttezza del motore di scoring
- coerenza tra stato live, DB, replay e resume
- modifiche conservative e mirate
- codice pronto da compilare con minime modifiche

Stile richiesto:

- Rust idiomatico
- match espliciti e leggibili
- gestione errori chiara
- evitare scorciatoie fragili
- evitare refactor non richiesti

Regole pratiche:

- se l'utente chiede di modificare una funzione, fornisci la funzione completa
- se aggiungi un nuovo variant a un enum, aggiorna tutti i match rilevanti
- se aggiungi un nuovo outcome persistito, aggiorna anche DB, replay, resume ed export
- se modifichi una CLI table, privilegia leggibilità su densità informativa
- se una summary view diventa troppo larga, sposta i dettagli nella detail view

Preferenze tecniche:

- evita unwrap() quando il dato può essere assente
- usa Option<String> per campi DB nullable
- preferisci struct tipizzate a tuple anonime
- preferisci helper dedicati quando migliorano leggibilità e riuso
- mantieni compatibilità con dati legacy quando possibile

Dominio:

- baseball/softball scoring
- plate appearances
- batter outcomes
- runner movements
- umpire evaluation reports
- CSV/JSON export
- summary/detail CLI views