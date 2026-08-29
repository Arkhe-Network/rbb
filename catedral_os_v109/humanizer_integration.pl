%%% ========================================================================
%%% SUBSTRATO 267 — HUMANIZER INTEGRATION LAYER
%%% ========================================================================
%%% Baseado em: blader/humanizer (GitHub)
%%% 35 padrões da Wikipedia: Signs of AI writing
%%% ========================================================================

:- module(humanizer_integration, [
    % --- Humanização ---
    humanize/2,                   % humanize(Text, Humanized)
    humanize_file/2,              % humanize_file(FilePath, OutputPath)
    detect_ai_patterns/2,         % detect_ai_patterns(Text, Patterns)
    pattern_count/2,              % pattern_count(Text, Count)

    % --- Padrões ---
    list_patterns/1,              % list_patterns(Patterns)
    get_pattern/2,                % get_pattern(Name, Pattern)

    % --- Integração com Substrato 233 (Research Agent) ---
    humanize_report/3,            % humanize_report(ReportID, Text, Humanized)

    % --- Integração com Substrato 263 (Synthetic User) ---
    humanize_interview/3,         % humanize_interview(InterviewID, Responses, Humanized)

    % --- Integração com Substrato 264 (Peer Review) ---
    humanize_review/3,            % humanize_review(ReviewID, Text, Humanized)

    % --- Testes ---
    run_humanizer_tests/0
]).

:- use_module(library(lists)).

:- use_module(library(apply)).

%%% ========================================================================
%%% 1. DEFINIÇÃO DOS 35 PADRÕES (SIMPLIFICADA)
%%% ========================================================================

pattern(1, 'Inflated importance', 'marking a pivotal moment in the evolution of', 'was established in 1989 as part of a wider decentralization').
pattern(2, 'Name-dropping', 'cited in NYT, BBC, FT, and The Hindu', 'Keep only useful, sourced context').
pattern(3, 'Shallow -ing analysis', 'symbolizing... reflecting... showcasing...', 'Keep only what the source supports').
pattern(4, 'Sales language', 'nestled within the breathtaking region', 'is a town in the Gonder region').
pattern(5, 'Vague sources', 'Experts believe it plays a crucial role', 'Name a real source or remove the claim').
pattern(6, 'Formulaic challenges', 'Despite challenges... continues to thrive', 'Keep the facts and remove the sales pitch').
pattern(7, 'Overused AI words', 'actually... additionally... showcasing...', 'also... needs... remain common').
pattern(8, 'Avoiding is/are', 'serves as... features... boasts', 'is... has').
pattern(9, 'Not X but Y', "It's not just X, it's Y", 'State the point directly').
pattern(10, 'Forced groups of three', 'innovation, inspiration, and insights', 'Use the number of items the meaning needs').
pattern(11, 'Changing names', 'protagonist... main character... hero', 'Use one name').
pattern(12, 'False X to Y ranges', 'from the Big Bang to dark matter', 'List the topics directly').
pattern(13, 'Passive voice', 'No configuration file needed', 'Name the actor when that helps').
pattern(14, 'Em/en dashes', 'institutions—not the people—yet this continues—', 'Cut them: periods, commas, colons').
pattern(15, 'Too much bold', '**OKRs**, **KPIs**, **BMC**', 'OKRs, KPIs, BMC').
pattern(16, 'Lists with bold headings', '**Performance:** Performance improved', 'Use prose when a list adds no value').
pattern(17, 'Title case in headings', 'Strategic Negotiations And Partnerships', 'Strategic negotiations and partnerships').
pattern(18, 'Emojis', '🚀 Launch Phase: Key Insight:', 'Remove emojis').
pattern(19, 'Curly quotes', 'said “the project”', 'said "the project"').
pattern(20, 'Chatbot text', 'I hope this helps! Let me know if...', 'Remove it').
pattern(21, 'Knowledge-limit disclaimers', 'While details are limited in available sources...', 'State what is known or remove the claim').
pattern(22, 'Overly agreeable tone', 'Great question! You''re absolutely right!', 'Answer directly').
pattern(23, 'Filler phrases', 'In order to', 'To').
pattern(24, 'Too many qualifiers', 'could potentially possibly', 'may').
pattern(25, 'Generic positive endings', 'The future looks bright', 'End with a fact or a sourced plan').
pattern(26, 'Too many hyphenated pairs', 'cross-functional, data-driven, client-facing', 'Keep only the hyphens grammar needs').
pattern(27, 'Fake deeper truth', 'At its core, what matters is...', 'State the point directly').
pattern(28, 'Announcing the next point', "Let's dive in", 'Start with the content').
pattern(29, 'Heading repeated below itself', '## Performance
Speed matters.', 'Let the heading do the work').
pattern(30, 'Writing about the old version', 'This function was added to replace...', 'Describe what it does now').
pattern(31, 'Forced punchlines', 'It had no preference. No prior. No nostalgia.', 'Use natural sentence lengths').
pattern(32, 'Formulaic sayings', 'Symmetry is the language of trust', 'State the specific claim').
pattern(33, 'Fake-candid openings', 'Honestly? It depends...', 'State the answer directly').
pattern(34, 'Answering objections no one raised', "This isn't mainly about prompt length...", 'Remove the unsupported defense').
pattern(35, 'Rejecting fake alternatives', 'A tempting option would be to..., but', 'Remove the fake option').

list_patterns(Patterns) :-
    findall(Name, pattern(_, Name, _, _), Patterns).

get_pattern(Name, Pattern) :-
    pattern(_, Name, Before, After),
    Pattern = pattern{name: Name, before: Before, after: After}.

%%% ========================================================================
%%% 2. DETECÇÃO E HUMANIZAÇÃO
%%% ========================================================================

detect_ai_patterns(Text, Patterns) :-
    findall(Name, (
        pattern(_, Name, Before, _),
        sub_string(Text, _, _, _, Before)
    ), Patterns).

pattern_count(Text, Count) :-
    detect_ai_patterns(Text, Patterns),
    length(Patterns, Count).

% Simplificação: substituição de padrões conhecidos
humanize(Text, Humanized) :-
    detect_ai_patterns(Text, Patterns),
    humanize_loop(Text, Patterns, Humanized).

humanize_loop(Text, [], Text).
humanize_loop(Text, [P|Rest], Result) :-
    pattern(_, P, Before, After),
    replace_substring(Text, Before, After, Temp),
    humanize_loop(Temp, Rest, Result).

replace_substring(String, Old, New, Result) :-
    split_string(String, Old, '', Parts),
    atomics_to_string(Parts, New, Result).

humanize_file(FilePath, OutputPath) :-
    read_file_to_string(FilePath, String, []),
    humanize(String, Humanized),
    open(OutputPath, write, Stream),
    write(Stream, Humanized),
    close(Stream).

%%% ========================================================================
%%% 3. INTEGRAÇÃO COM SUBSTRATOS
%%% ========================================================================

% Integração com Research Agent (233)
humanize_report(ReportID, Text, Humanized) :-
    humanize(Text, Humanized),
    format('[HUMANIZER] Relatório ~w humanizado~n', [ReportID]).

% Integração com Synthetic User (263)
humanize_interview(InterviewID, Responses, Humanized) :-
    humanize(Responses, Humanized),
    format('[HUMANIZER] Entrevista ~w humanizada~n', [InterviewID]).

% Integração com Peer Review (264)
humanize_review(ReviewID, Text, Humanized) :-
    humanize(Text, Humanized),
    format('[HUMANIZER] Parecer ~w humanizado~n', [ReviewID]).

%%% ========================================================================
%%% 4. TESTES
%%% ========================================================================

run_humanizer_tests :-
    format('~n╔═══════════════════════════════════════════════════════════════╗~n'),
    format('║  🧬 SUBSTRATO 267 — HUMANIZER INTEGRATION                  ║~n'),
    format('╚═══════════════════════════════════════════════════════════════╝~n'),

    format('~n─── [1] Lista de Padrões ───~n'),
    list_patterns(Patterns),
    format('  Total: ~w padrões~n', [length(Patterns)]),

    format('~n─── [2] Detecção de Padrões ───~n'),
    Text = 'This marks a pivotal moment in the evolution of AI, showcasing innovation.',
    detect_ai_patterns(Text, Found),
    format('  Padrões encontrados: ~w~n', [Found]),

    format('~n─── [3] Humanização ───~n'),
    humanize(Text, Humanized),
    format('  Original: ~w~n', [Text]),
    format('  Humanizado: ~w~n', [Humanized]),

    format('~n╔═══════════════════════════════════════════════════════════════╗~n'),
    format('║  ✅ SUBSTRATO 267 — TESTES CONCLUÍDOS                      ║~n'),
    format('╚═══════════════════════════════════════════════════════════════╝~n').

:- initialization(run_humanizer_tests, main).
