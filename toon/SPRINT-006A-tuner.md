## SPRINT-006A — Tuner & Surface de recherche

### Tâches
- [ ] Créer `crates/tuner` (lib): impl `TuningSpace`, `TuningStrategy`, `EpisodeRunner`, `Score`.
- [ ] Ajouter au `cli` un mode `--mode tune` (offline/online).
- [ ] Lire l’espace de recherche depuis `config/tuning.toml` + overrides **ENV** (injectés par `run_bot_mainnet.sh`).
- [ ] Impl stratégies: **grid**, **random**, **epsilon-greedy bandit**, **bayes** (naïf, ex. TPE-like).
- [ ] Offline seeding: consommer Parquet de replay → estimer Score pour K configs seed.
- [ ] Online loop: épisode T minutes, exé safe (caps), collecte métriques → Score → update prior.
- [ ] Early-stop + rollback vers best config; persist `config/tuned/*.toml`.
- [ ] Exposer métriques tuning dans CSV/JSON + petit récap CLI.

### Interfaces (Rust)
- `TuningParam { key: &'static str, default: f64|i64|enum, bounds: (min,max), step: Option<f64> }`
- `TuningSpace { params: Vec<TuningParam> }`
- `TuningStrategy::next_config(&self, history: &[EpisodeResult]) -> Config`
- `EpisodeRunner::run(config) -> EpisodeResult { pnl_bps, hit_rate, slippage_p95, latency_ms, risk_events }`
- `Score::from(metrics, constraints, lambdas) -> f64`

### Tests
- Unit: scoring, sampling borné, epsilon-greedy explore/exploit, rollback.
- Intégration: offline seeding (Parquet mock), 10 épisodes online (paper), persist best config.

### DoD
- ✅ `toon --mode tune` tourne offline+online (paper) sans erreurs.
- ✅ Résultats JSON + tuned TOML présents.
- ✅ Score ≥ baseline (ou slippage p95 en baisse).
