# Synoema: Научные основания проектирования языка программирования для LLM

**Версия:** 0.2 — Март 2026
**Статус:** Верифицированная научная база

---

## Аннотация

Данный документ систематизирует проверяемые научные факты, подтверждающие целесообразность создания нового языка программирования, оптимизированного для генерации кода большими языковыми моделями (LLM). Каждое утверждение сопровождается ссылкой на рецензируемую публикацию или воспроизводимый эксперимент.

---

## I. Фундаментальная проблема: стоимость токена

### 1.1 Экономика inference

Inference (генерация текста) — не training — определяет операционную стоимость LLM-систем.

**Факт 1.1.** Inference потребляет более 90% общего энергопотребления LLM за жизненный цикл.
— *Источник:* AWS operational lifecycle report; подтверждено в TokenPowerBench (arxiv:2512.03024, декабрь 2025): "inference, not training, accounts for more than 90% of total power consumption."

**Факт 1.2.** Рынок LLM inference оценивается в $106 млрд в 2025 и прогнозируется $250+ млрд к 2030 (CAGR 19.2%).
— *Источник:* MarketsandMarkets, 2025; цитируется в TokenPowerBench.

**Факт 1.3.** Стоимость одного токена у OpenAI: ~$0.00012 (при масштабах их инфраструктуры); для типичных организаций — до $0.001 за токен.
— *Источник:* Introl.com cost analysis, январь 2026.

**Следствие:** Любое сокращение количества токенов для эквивалентной семантики напрямую сокращает операционные расходы.

### 1.2 Квадратичная стоимость внимания

**Факт 1.4.** Вычислительная сложность self-attention в стандартном трансформере — O(n² · d), где n — длина последовательности, d — размерность модели. Удвоение длины контекста требует четырёхкратных вычислительных ресурсов.
— *Источник:* Vaswani et al. "Attention Is All You Need" (NeurIPS 2017); подтверждено в Context Engineering (Medium, октябрь 2025): "doubling the context length results in four times the computational requirements."

**Следствие (критически важно для Synoema):** Сокращение длины последовательности на k% даёт сокращение стоимости attention на ~(1-(1-k/100)²)×100%. При k=50% (50% меньше токенов) — экономия до 75% вычислений attention.

### 1.3 Влияние длины контекста на качество

**Факт 1.5.** Качество ответов LLM деградирует с ростом длины входного контекста ("context rot"). Модели не используют контекст равномерно — производительность становится всё менее надёжной по мере увеличения длины ввода.
— *Источник:* Hong, Troynikov, Huber. "Context Rot: How Increasing Input Tokens Impacts LLM Performance" (Chroma Research, июль 2025): "models do not use their context uniformly; instead, their performance grows increasingly unreliable as input length grows."

**Следствие:** Более компактный код не только дешевле — он потенциально лучше обрабатывается моделью.

### 1.4 Двухфазная латентность inference

**Факт 1.6.** Inference состоит из двух фаз: prefill (параллельная обработка входных токенов) и decode (последовательная генерация выходных токенов). Каждый выходной токен добавляет от нескольких до десятков миллисекунд латентности.
— *Источник:* Redis LLM Token Optimization (февраль 2026); CLONE (USENIX ATC 2025): "an increase in generated tokens significantly impacts overall latency."

**Факт 1.7.** KV-кэш растёт линейно с числом обрабатываемых токенов и является основным ограничением пропускной способности при масштабировании.
— *Источник:* Redis (2026); Kwon et al. "PagedAttention" (SOSP 2023).

---

## II. Токенная эффективность языков программирования

### 2.1 Измеренные различия между языками

**Факт 2.1.** Разница в токенной эффективности между языками программирования достигает 2.6x для эквивалентных программ. J (array language, ASCII) — лидер (~70 токенов в среднем), C — аутсайдер (~182 токена).
— *Источник:* Martin Alderson. "Which programming languages are most token-efficient?" (январь 2026). Вошёл в топ Hacker News.

**Факт 2.2.** APL, несмотря на крайнюю краткость исходного кода, проиграл J из-за специальных символов (⍳, ⍴, ⌽), которые BPE-токенизатор кодирует множественными токенами.
— *Источник:* Alderson (2026): "APL's famous terseness isn't a plus for LLMs: the tokenizer is badly optimised for its symbol set, so all those unique glyphs end up as multiple tokens each."

**Критический вывод для Synoema:** Эффективность зависит не от краткости исходного кода в символах, а от того, как эти символы ложатся на BPE-словарь. ASCII-символы, часто встречающиеся в обучающих данных, кодируются эффективнее.

**Факт 2.3.** Функциональные языки с type inference (Haskell ~115 tok, F# ~118 tok) сопоставимы по токенной эффективности с динамическими языками (Python ~130 tok), но обеспечивают compile-time гарантии.
— *Источник:* Alderson (2026); подтверждено дискуссией на HN: "Haskell and F# were barely less efficient than the most efficient dynamic languages... typed languages for LLMs has an awful lot of benefits."

### 2.2 Токенная эффективность форматов данных

**Факт 2.4.** TOON (Token-Oriented Object Notation) обеспечивает 40-60% сокращение токенов по сравнению с JSON при сохранении или улучшении accuracy LLM-понимания данных.
— *Источник:* TOON benchmark (2025), протестирован на 209 вопросах и 4 моделях: TOON 76.4% accuracy при 2,759 токенах vs JSON 73.7% accuracy при 3,104 токенах. InfoQ coverage (ноябрь 2025).

**Факт 2.5.** TOON достигает 99.4% accuracy на GPT-5 Nano при 46% меньшем количестве токенов по сравнению с JSON.
— *Источник:* InfoQ (ноябрь 2025): "TOON hits 99.4% accuracy on GPT 5 Nano while using 46% fewer tokens."

**Факт 2.6.** 40-70% токенов в JSON-данных — структурный "шум" (скобки, кавычки, повторяющиеся ключи), не несущий семантической информации.
— *Источник:* TOON vs JSON analysis (tensorlake.ai, декабрь 2025); New Stack guide (декабрь 2025): "Poor data serialization consumes 40% to 70% of available tokens through unnecessary formatting overhead."

---

## III. BPE-токенизация: формальные свойства

### 3.1 Механизм BPE

**Факт 3.1.** BPE (Byte Pair Encoding) — жадный алгоритм, итеративно объединяющий наиболее частые пары смежных токенов. Для LLM используется модифицированная версия, оптимизирующая не максимальное сжатие, а эффективное представление.
— *Источник:* Gage (1994); модификация описана в Sennrich et al. (2016) "Neural Machine Translation of Rare Words with Subword Units"; Wikipedia BPE article.

**Факт 3.2.** Словарь GPT-4/GPT-4o (cl100k_base): 100,258 токенов (100,000 BPE + 258 специальных). Byte-level BPE гарантирует кодирование любого UTF-8 текста.
— *Источник:* Wikipedia BPE; OpenAI tiktoken documentation.

**Факт 3.3.** BPE-токенизация доказуемо детерминистична с конечным опережающим просмотром, и может быть представлена детерминированным конечным автоматом (DFA).
— *Источник:* Berglund & van der Merwe (2023) "Formalizing BPE Tokenization"; Berglund et al. (2024) — DFA-конструкция.

### 3.2 Проблема неполных токенов

**Факт 3.4.** Некорректные слияния BPE могут пересекать границы UTF-8 символов, создавая "неполные токены" (stray bytes), коррелирующие с повышенной вероятностью галлюцинаций.
— *Источник:* Jang et al. (2024); Land et al. (май 2025); цитируется в Emergent Mind BPE survey: "improbable bigram constructions spanning script boundaries [correlate] with elevated hallucination rates."

### 3.3 Продвинутые варианты BPE

**Факт 3.5.** SuperBPE, снимающий ограничение на слияния через пробельные границы, достигает до 33% сокращения токенов при том же словаре и +4% прироста accuracy при 27% снижении FLOPs на байт.
— *Источник:* Liu et al. (март 2025); цитируется в Emergent Mind: "up to 33% fewer tokens and 27% less FLOPs per byte."

---

## IV. Constrained Decoding: от синтаксиса к типам

### 4.1 Основная идея

**Факт 4.1.** Constrained decoding гарантирует 100% синтаксическую корректность вывода LLM, маскируя невалидные токены на каждом шаге декодирования.
— *Источник:* Willard & Louf (2023) "Efficient Guided Generation for LLMs"; XGrammar (Dong et al., 2024).

### 4.2 Проблема искажения распределения

**Факт 4.2 (критически важный).** Наивное constrained decoding искажает вероятностное распределение модели, приводя к выводу с высокой perplexity и снижению downstream accuracy. Причина — несовпадение BPE-токенов с терминалами грамматики (token misalignment).
— *Источник:* Beurer-Kellner, Fischer, Vechev. "Guiding LLMs The Right Way: Fast, Non-Invasive Constrained Generation" (ICML 2024, Domino): "misalignment can lead to a significant decrease in downstream accuracy."

**Факт 4.3.** Grammar-Aligned Decoding (NeurIPS 2024) формально доказывает, что стандартные GCD-техники порождают выходы с вероятностями, непропорциональными модельному распределению, и предлагает ASAp — алгоритм, сохраняющий условное распределение.
— *Источник:* Park et al. "Grammar-Aligned Decoding" (NeurIPS 2024 poster): "GCD techniques can distort the LLM's distribution, leading to outputs that are grammatical but... low-quality."

### 4.3 Производительность XGrammar

**Факт 4.4.** XGrammar достигает до 100x ускорения по сравнению с предыдущими решениями и обеспечивает near-zero overhead при генерации JSON.
— *Источник:* Dong et al. "XGrammar: Flexible and Efficient Structured Generation Engine" (2024). Является стандартом де-факто для vLLM, SGLang, TensorRT-LLM, MLC-LLM.

**Факт 4.5.** Compressed Finite State Machine (SGLang) позволяет декодировать несколько токенов за один шаг на детерминированных участках грамматики (jump-forward decoding).
— *Источник:* Zheng et al. "SGLang: Efficient Execution of Structured Language Model Programs" (NeurIPS 2024).

### 4.4 Детерминированные грамматики и замкнутые формы

**Факт 4.6.** Для регулярных и детерминированных контекстно-свободных языков ограничения (constraints) могут быть скомпилированы в замкнутой форме с порядками величин более быстрой предварительной обработкой.
— *Источник:* Tian et al. "Automata-Based Constraints for Language Model Decoding" (CoLM 2024): "for regular and deterministic context-free languages, constraints can be compiled in closed form with orders-of-magnitude faster setup."

**Вывод для Synoema:** Детерминированная контекстно-свободная грамматика (DCFG) — оптимальный выбор для языка, ориентированного на constrained decoding.

---

## V. Типовые системы против галлюцинаций в коде

### 5.1 Масштаб проблемы

**Факт 5.1.** Типовые ошибки составляют 33.6% всех отказов LLM-генерируемых программ.
— *Источник:* Tambon et al. (2025); Dou et al. (2024); цитируется в "Learning to Guarantee Type Correctness in Code Generation" (arxiv:2510.10216): "type errors alone account for 33.6% of failed LM-generated programs."

**Факт 5.2.** 24% предложений GitHub Copilot на LeetCode-задачах содержали ошибки компиляции, основная причина — типовые ошибки.
— *Источник:* Nguyen & Nadi (2022); Mündler et al. (2025).

### 5.2 Type-constrained decoding: доказанная эффективность

**Факт 5.3 (ключевой).** Type-constrained decoding для TypeScript сокращает ошибки компиляции более чем вдвое и значительно повышает функциональную корректность для синтеза, трансляции и ремонта кода.
— *Источник:* Mündler et al. "Type-Constrained Code Generation with Language Models" (PLDI 2025, ACM SIGPLAN): "reduces compilation errors by more than half and significantly increases functional correctness in code synthesis, translation, and repair tasks across LLMs of various sizes and model families."

**Факт 5.4.** На HumanEval type-constraining сокращает ошибки компиляции на 74.8%, тогда как синтаксическое ограничение даёт лишь 9.0% — тройка порядка разницы.
— *Источник:* Mündler et al. (PLDI 2025), Table 2: "Type constraining reduces compiler errors by 74.8% in the synthesis of HumanEval problems, compared to only 9.0% ideal improvement through syntax-only constraining."

**Факт 5.5.** Синтаксические ограничения в чистом виде дают минимальный эффект, потому что синтаксические ошибки и так редки в LLM-генерируемом коде. Основная проблема — семантическая корректность, обеспечиваемая типами.
— *Источник:* Mündler et al. (PLDI 2025): "syntax accounts for only a small part of overall program correctness"; Poesia et al. (2022).

### 5.3 Типы как обратная связь для LLM

**Факт 5.6.** TypeScript-типы действуют как "жёсткие ограничения и высококачественные промпты" для AI-ассистентов, уменьшая вероятность галлюцинаций. AI-редакторы используют TypeScript LSP как валидационную петлю.
— *Источник:* Avetisyan (Medium, ноябрь 2025): "interface and type definitions act as hard constraints and high-quality prompts... dramatically reduce hallucination rates"; Landgraf (Medium, октябрь 2025): "90% reduction in ID mix-up bugs — branded types catch them at compile time; 3x faster LLM convergence."

### 5.4 Hindley-Milner как оптимальная система типов для Synoema

**Факт 5.7.** Hindley-Milner (HM) type inference позволяет выводить типы автоматически, без аннотаций. Это критически важно: аннотации стоят токенов, а HM получает те же гарантии без дополнительных токенов.
— *Источник:* Alderson (2026) объясняет парадокс: Haskell и F# с type inference столь же токен-эффективны, как Python, но дают compile-time гарантии.

**Факт 5.8.** Тип-направленный синтез программ (type-guided program synthesis) использует типовые ограничения в виде ограниченных клаузов Хорна (constrained Horn clauses), что позволяет включить произвольные разрешимые свойства в систему типов.
— *Источник:* "Learning to Guarantee Type Correctness in Code Generation" (arxiv:2510.10216, октябрь 2025): "all typing rules are formulated as constrained Horn clauses... can accommodate a broader class of Turing-computable specifications."

---

## VI. BPE-Aligned Grammar Design: центральная инновация Synoema

### 6.1 Проблема misalignment (формализация)

Пусть G — грамматика языка L с множеством терминалов Σ_G.
Пусть T — BPE-токенизатор с словарём V_T.
Пусть σ ∈ Σ_G — терминал грамматики, и τ = T(σ) — его токенизация.

**Misalignment** возникает когда ∃σ ∈ Σ_G : |T(σ)| > 1, то есть один терминал грамматики разбивается на множественные BPE-токены.

**Факт 6.1 (формализован в Domino).** Когда LLM генерирует "bridge token" (токен, охватывающий границу нескольких терминалов грамматики), наивное constrained decoding вынуждено либо отвергнуть валидный bridge token, либо принять невалидный; оба случая искажают распределение.
— *Источник:* Domino (Beurer-Kellner et al., ICML 2024).

### 6.2 Условие BPE-выравнивания

**Определение.** Грамматика G является BPE-выровненной относительно токенизатора T, если для каждого терминала σ ∈ Σ_G выполняется одно из условий:
1. |T(σ)| = 1 (терминал — ровно один BPE-токен), или
2. σ — конкатенация элементов, каждый из которых является одним BPE-токеном.

**Теоретическое следствие:** Для BPE-выровненной DCFG, constrained decoding не создаёт bridge tokens, устраняя misalignment и его негативные эффекты на accuracy.

### 6.3 Верификация выравнивания для Synoema

Все ключевые операторы Synoema верифицированы на совпадение с одиночными BPE-токенами в cl100k_base (GPT-4) и Llama tokenizer:

| Оператор | Символ | cl100k_base | Llama |
|----------|--------|-------------|-------|
| Комментарий | `--` | 1 токен | 1 токен |
| Стрелка | `->` | 1 токен | 1 токен |
| Лямбда | `\` | 1 токен | 1 токен |
| Условие | `?` | 1 токен | 1 токен |
| Тип-аннотация | `:` | 1 токен | 1 токен |
| Доступ к полю | `.` | 1 токен | 1 токен |
| Bind (эффект) | `<-` | 1 токен | 1 токен |
| Присваивание | `=` | 1 токен | 1 токен |
| Конкатенация | `++` | 1 токен | 1 токен |
| Директива | `@` | 1 токен | 1 токен |
| Пайп | `|>` | 1-2 токена | 1-2 токена |

Верификация возможна программно через tiktoken / sentencepiece.

---

## VII. Архитектурное обоснование

### 7.1 Позиция в иерархии Хомского

| Класс грамматик | Распознаватель | Constrained decoding overhead | Synoema |
|----------------|---------------|-------------------------------|-------|
| Регулярные (Type 3) | DFA | O(1), precomputable | Уровень лексера |
| **DCFG** | DPDA | **Замкнутая форма** (CoLM 2024) | **Уровень парсера** |
| КС (Type 2) | PDA | Near-zero (XGrammar) | Fallback |
| КЗ (Type 1) | LBA | Дорого, только эвристики | Система типов |

### 7.2 Компиляция грамматики

Цепочка компиляции:
```
Synoema DCFG → DPDA → XGrammar Token Mask Cache → Near-zero overhead decoding
```

Детерминированные участки (ключевые слова, обязательные разделители) генерируются через jump-forward decoding без обращения к LLM (SGLang compressed FSM).

### 7.3 Type-guided constraining

Synoema объединяет два уровня:
1. **Синтаксический** (DCFG → FSM): гарантирует грамматическую корректность
2. **Типовой** (HM inference → prefix automata): гарантирует типовую корректность

Это даёт эффект, измеренный Mündler et al.: 74.8% vs 9.0% сокращения ошибок компиляции (тип-constraining vs syntax-only).

---

## VIII. Открытые вопросы

### 8.1 Cold start проблема
LLM обучены преимущественно на Python/JS/Java. Новый язык не представлен в обучающих данных. Необходимы исследования:
- Few-shot эффективность генерации на новых языках
- Минимальный объём fine-tuning данных
- Transfer learning с Haskell/F#/Clojure как ближайших языков

### 8.2 Индентация vs фиксированные разделители
Индентация (Python/YAML подход) экономит скобки, но пробельные символы тоже стоят токенов. Необходим эмпирический эксперимент на реальных BPE-токенизаторах для определения оптимума.

### 8.3 Effect system
Выбор между монадами (Haskell), algebraic effects (Eff/Koka) и простой IO-маркировкой (@io) влияет на токенную эффективность. Необходимы бенчмарки.

### 8.4 Row polymorphism overhead
Row types дают гибкость при работе с записями, но усложняют type inference и увеличивают размер типовых ограничений, передаваемых в constrained decoding engine. Необходима оценка trade-off.

---

## IX. Библиография (верифицированные источники)

### Токенная эффективность
1. Alderson, M. (2026). "Which programming languages are most token-efficient?" martinalderson.com
2. TOON Format (2025). toon-format/toon. GitHub. Benchmarks on 209 questions, 4 models.
3. TOON coverage: InfoQ (ноябрь 2025). "New TOON Hopes to Cut LLM Costs."

### BPE и токенизация
4. Gage, P. (1994). "A New Algorithm for Data Compression." C Users Journal 12(2).
5. Sennrich, R. et al. (2016). "Neural Machine Translation of Rare Words with Subword Units." ACL.
6. Berglund & van der Merwe (2023). "Formalizing BPE Tokenization." ResearchGate.
7. Liu et al. (март 2025). SuperBPE: "up to 33% fewer tokens." Emergent Mind summary.
8. Land et al. (май 2025). SCRIPT-BPE. Emergent Mind Byte-level BPE survey.

### Constrained Decoding
9. Willard, B.T. & Louf, R. (2023). "Efficient Guided Generation for LLMs." Outlines.
10. Dong et al. (2024). "XGrammar: Flexible and Efficient Structured Generation Engine." arXiv:2411.15100.
11. Zheng et al. (2024). "SGLang: Efficient Execution of Structured Language Model Programs." NeurIPS 2024.
12. Beurer-Kellner et al. (2024). "Domino: Fast, Non-Invasive Constrained Generation." ICML 2024.
13. Park et al. (2024). "Grammar-Aligned Decoding." NeurIPS 2024.
14. Tian et al. (2024). "Automata-Based Constraints for Language Model Decoding." CoLM 2024.

### Типовые системы и корректность кода
15. Mündler et al. (2025). "Type-Constrained Code Generation with Language Models." PLDI 2025 (ACM SIGPLAN). arXiv:2504.09246.
16. Tambon et al. (2025). "Bugs in LLM generated code: an empirical study." Empir. Softw. Eng.
17. "Learning to Guarantee Type Correctness in Code Generation." arXiv:2510.10216.
18. Zhang et al. (2024). "LLM Hallucinations in Practical Code Generation." arXiv:2409.20550.

### Inference стоимость и латентность
19. TokenPowerBench (2025). arXiv:2512.03024.
20. Hong et al. (2025). "Context Rot." Chroma Research.
21. Vaswani et al. (2017). "Attention Is All You Need." NeurIPS.
22. Kwon et al. (2023). "PagedAttention." SOSP.
23. TALE (2025). "Token-Budget-Aware LLM Reasoning." ACL 2025 Findings.

---

*Документ подготовлен как научная база проекта Synoema.*
*Все факты верифицируемы через указанные источники.*
*Последнее обновление: март 2026.*
