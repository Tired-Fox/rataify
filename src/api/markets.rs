use super::IntoSpotifyParam;

/// Available markets: https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2
#[derive(Debug, Clone, Copy, PartialEq, Hash, PartialOrd, strum_macros::Display)]
pub enum Market {
    /// Andorra  (1974)
    AD,
    /// United Arab Emirates  (1974)
    AE,
    /// Afghanistan  (1974)
    AF,
    /// Antigua and Barbuda  (1974)
    AG,
    /// Anguilla  (1985) AI previously represented French Afars and Issas
    AI,
    /// Albania  (1974)
    AL,
    /// Armenia  (1992)
    AM,
    /// Angola  (1974)
    AO,
    /// Antarctica  (1974) Covers the territories south of 60° south latitude
    AQ,

    /// Argentina  (1974)
    AR,
    /// American Samoa  (1974)
    AS,
    /// Austria  (1974)
    AT,
    /// Australia  (1974) Includes the Ashmore and Cartier Islands and the Coral Sea Islands
    AU,
    /// Aruba  (1986)
    AW,
    /// Åland Islands  (2004) An autonomous county of Finland
    AX,
    /// Azerbaijan  (1992)
    AZ,
    /// Bosnia and Herzegovina  (1992)
    BA,
    /// Barbados  (1974)
    BB,
    /// Bangladesh  (1974)
    BD,
    /// Belgium  (1974)
    BE,
    /// Burkina Faso  (1984) Name changed from Upper Volta (HV)
    BF,
    /// Bulgaria  (1974)
    BG,
    /// Bahrain  (1974)
    BH,
    /// Burundi  (1974)
    BI,
    /// Benin  (1977) Name changed from Dahomey (DY)
    BJ,
    /// Saint Barthélemy  (2007)
    BL,
    /// Bermuda  (1974)
    BM,
    /// Brunei Darussalam  (1974) Previous ISO country name: Brunei
    BN,
    /// Bolivia, Plurinational State of  (1974) Previous ISO country name: Bolivia
    BO,
    /// Bonaire, Sint Eustatius and Saba  (2010) Consists of three Caribbean "special municipalities", which are part of the Netherlands proper: Bonaire, Sint Eustatius, and Saba (the BES Islands)
    BQ,

    /// Brazil  (1974)
    BR,
    /// Bahamas  (1974)
    BS,
    /// Bhutan  (1974)
    BT,
    /// Bouvet Island  (1974) Dependency of Norway
    BV,
    /// Botswana  (1974)
    BW,
    /// Belarus  (1974) Code taken from previous ISO country name: Byelorussian SSR (now assigned ISO 3166-3 code BYAA)
    BY,

    /// Belize  (1974)
    BZ,
    /// Canada  (1974)
    CA,
    /// Cocos (Keeling) Islands  (1974) External territory of Australia
    CC,
    /// Congo, Democratic Republic of the  (1997) Name changed from Zaire (ZR)
    CD,
    /// Central African Republic  (1974)
    CF,
    /// Congo  (1974)
    CG,
    /// Switzerland  (1974) Code taken from name in Latin: Confoederatio Helvetica
    CH,
    /// Côte d'Ivoire  (1974) ISO country name follows UN designation (common name and previous ISO country name: Ivory Coast)
    CI,
    /// Cook Islands  (1974)
    CK,
    /// Chile  (1974)
    CL,
    /// Cameroon  (1974) Previous ISO country name: Cameroon, United Republic of
    CM,
    /// China  (1974)
    CN,
    /// Colombia  (1974)
    CO,
    /// Costa Rica  (1974)
    CR,
    /// Cuba  (1974)
    CU,
    /// Cabo Verde  (1974) ISO country name follows UN designation (common name and previous ISO country name: Cape Verde, another previous ISO country name: Cape Verde Islands)
    CV,
    /// Curaçao  (2010)
    CW,
    /// Christmas Island  (1974) External territory of Australia
    CX,
    /// Cyprus  (1974)
    CY,
    /// Czechia  (1993) Previous ISO country name: Czech Republic
    CZ,
    /// Germany  (1974) Code taken from name in German: Deutschland
    DE,

    /// Djibouti  (1977) Name changed from French Afars and Issas (AI)
    DJ,
    /// Denmark  (1974)
    DK,
    /// Dominica  (1974)
    DM,
    /// Dominican Republic  (1974)
    DO,
    /// Algeria  (1974) Code taken from name in Arabic الجزائر al-Djazā'ir, Algerian Arabic الدزاير al-Dzāyīr, or Berber ⴷⵣⴰⵢⵔ Dzayer
    DZ,
    /// Ecuador  (1974)
    EC,
    /// Estonia  (1992) Code taken from name in Estonian: Eesti
    EE,
    /// Egypt  (1974)
    EG,
    /// Western Sahara  (1974) Previous ISO country name: Spanish Sahara (code taken from name in Spanish: Sahara español) .eh ccTLD has not been implemented.[15]
    EH,
    /// Eritrea  (1993)
    ER,
    /// Spain  (1974) Code taken from name in Spanish: España
    ES,
    /// Ethiopia  (1974)
    ET,
    /// Finland  (1974)
    FI,
    /// Fiji  (1974)
    FJ,
    /// Falkland Islands (Malvinas)  (1974) ISO country name follows UN designation due to the Falkland Islands sovereignty dispute (local common name: Falkland Islands)[16]
    FK,
    /// Micronesia, Federated States of  (1986) Previous ISO country name: Micronesia
    FM,
    /// Faroe Islands  (1974) Code taken from name in Faroese: Føroyar
    FO,
    /// France  (1974) Includes Clipperton Island
    FR,
    /// Gabon  (1974)
    GA,
    /// United Kingdom of Great Britain and Northern Ireland  (1974) Includes Akrotiri and Dhekelia (Sovereign Base Areas) Code taken from Great Britain (from official name: United Kingdom of Great Britain and Northern Ireland)[17] Previous ISO country name: United Kingdom .uk is the primary ccTLD of the United Kingdom instead of .gb (see code UK, which is exceptionally reserved)
    GB,
    /// Grenada  (1974)
    GD,
    /// Georgia  (1992) GE previously represented Gilbert and Ellice Islands
    GE,
    /// French Guiana  (1974) Code taken from name in French: Guyane française
    GF,
    /// Guernsey  (2006) A British Crown Dependency
    GG,
    /// Ghana  (1974)
    GH,
    /// Gibraltar  (1974)
    GI,
    /// Greenland  (1974)
    GL,
    /// Gambia  (1974)
    GM,
    /// Guinea  (1974)
    GN,
    /// Guadeloupe  (1974)
    GP,
    /// Equatorial Guinea  (1974) Code taken from name in French: Guinée équatoriale
    GQ,
    /// Greece  (1974)
    GR,
    /// South Georgia and the South Sandwich Islands  (1993)
    GS,
    /// Guatemala  (1974)
    GT,
    /// Guam  (1974)
    GU,
    /// Guinea-Bissau  (1974)
    GW,
    /// Guyana  (1974)
    GY,
    /// Hong Kong  (1974) Hong Kong is officially a Special Administrative Region of the People's Republic of China since 1 July 1997
    HK,
    /// Heard Island and McDonald Islands  (1974) External territory of Australia
    HM,
    /// Honduras  (1974)
    HN,
    /// Croatia  (1992) Code taken from name in Croatian: Hrvatska
    HR,
    /// Haiti  (1974)
    HT,
    /// Hungary  (1974)
    HU,
    /// Indonesia  (1974)
    ID,
    /// Ireland  (1974)
    IE,
    /// Israel  (1974)
    IL,
    /// Isle of Man  (2006) A British Crown Dependency
    IM,
    /// India  (1974)
    IN,
    /// British Indian Ocean Territory  (1974)
    IO,
    /// Iraq  (1974)
    IQ,
    /// Iran, Islamic Republic of  (1974) Previous ISO country name: Iran
    IR,
    /// Iceland  (1974) Code taken from name in Icelandic: Ísland
    IS,
    /// Italy  (1974)
    IT,
    /// Jersey  (2006) A British Crown Dependency
    JE,
    /// Jamaica  (1974)
    JM,
    /// Jordan  (1974)
    JO,
    /// Japan  (1974)
    JP,
    /// Kenya  (1974)
    KE,
    /// Kyrgyzstan  (1992)
    KG,
    /// Cambodia  (1974) Code taken from former name: Khmer Republic
    KH,

    /// Kiribati  (1979) Name changed from Gilbert Islands (GE)
    KI,
    /// Comoros  (1974) Code taken from name in Comorian: Komori
    KM,

    /// Saint Kitts and Nevis  (1974) Previous ISO country name: Saint Kitts-Nevis-Anguilla
    KN,
    /// Korea, Democratic People's Republic of  (1974) ISO country name follows UN designation (common name: North Korea)
    KP,
    /// Korea, Republic of  (1974) ISO country name follows UN designation (common name: South Korea)
    KR,
    /// Kuwait  (1974)
    KW,
    /// Cayman Islands  (1974)
    KY,
    /// Kazakhstan  (1992) Previous ISO country name: Kazakstan
    KZ,
    /// Lao People's Democratic Republic  (1974) ISO country name follows UN designation (common name and previous ISO country name: Laos)
    LA,
    /// Lebanon  (1974)
    LB,
    /// Saint Lucia  (1974)
    LC,
    /// Liechtenstein  (1974)
    LI,
    /// Sri Lanka  (1974)
    LK,
    /// Liberia  (1974)
    LR,
    /// Lesotho  (1974)
    LS,
    /// Lithuania  (1992) LT formerly reserved indeterminately for Libya Tripoli
    LT,
    /// Luxembourg  (1974)
    LU,
    /// Latvia  (1992)
    LV,
    /// Libya  (1974) Previous ISO country name: Libyan Arab Jamahiriya
    LY,
    /// Morocco  (1974) Code taken from name in French: Maroc
    MA,
    /// Monaco  (1974)
    MC,
    /// Moldova, Republic of  (1992) Previous ISO country name: Moldova (briefly from 2008 to 2009)
    MD,
    /// Montenegro  (2006) ME formerly reserved indeterminately for Western Sahara
    ME,
    /// Saint Martin (French part)  (2007) The Dutch part of Saint Martin island is assigned code SX
    MF,
    /// Madagascar  (1974)
    MG,
    /// Marshall Islands  (1986)
    MH,
    /// North Macedonia  (1993) Code taken from name in Macedonian: Severna Makedonija
    MK,

    /// Mali  (1974)
    ML,
    /// Myanmar  (1989) Name changed from Burma (BU)
    MM,
    /// Mongolia  (1974)
    MN,
    /// Macao  (1974) Previous ISO country name: Macau; Macao is officially a Special Administrative Region of the People's Republic of China since 20 December 1999
    MO,
    /// Northern Mariana Islands  (1986)
    MP,
    /// Martinique  (1974)
    MQ,
    /// Mauritania  (1974)
    MR,
    /// Montserrat  (1974)
    MS,
    /// Malta  (1974)
    MT,
    /// Mauritius  (1974)
    MU,
    /// Maldives  (1974)
    MV,
    /// Malawi  (1974)
    MW,
    /// Mexico  (1974)
    MX,
    /// Malaysia  (1974)
    MY,
    /// Mozambique  (1974)
    MZ,
    /// Namibia  (1974)
    NA,
    /// New Caledonia  (1974)
    NC,
    /// Niger  (1974)
    NE,
    /// Norfolk Island  (1974) External territory of Australia
    NF,
    /// Nigeria  (1974)
    NG,
    /// Nicaragua  (1974)
    NI,
    /// Netherlands, Kingdom of the  (1974) Officially includes the islands Bonaire, Saint Eustatius and Saba, which also have code BQ in ISO 3166-1. Within ISO 3166-2, Aruba (AW), Curaçao (CW), and Sint Maarten (SX) are also coded as subdivisions of NL.[18]
    NL,

    /// Norway  (1974)
    NO,
    /// Nepal  (1974)
    NP,
    /// Nauru  (1974)
    NR,
    /// Niue  (1974) Previous ISO country name: Niue Island
    NU,
    /// New Zealand  (1974)
    NZ,
    /// Oman  (1974)
    OM,
    /// Panama  (1974)
    PA,
    /// Peru  (1974)
    PE,
    /// French Polynesia  (1974) Code taken from name in French: Polynésie française
    PF,
    /// Papua New Guinea  (1974)
    PG,
    /// Philippines  (1974)
    PH,
    /// Pakistan  (1974)
    PK,
    /// Poland  (1974)
    PL,
    /// Saint Pierre and Miquelon  (1974)
    PM,
    /// Pitcairn  (1974) Previous ISO country name: Pitcairn Islands
    PN,
    /// Puerto Rico  (1974)
    PR,
    /// Palestine, State of  (1999) Previous ISO country name: Palestinian Territory, Occupied
    PS,

    /// Portugal  (1974)
    PT,
    /// Palau  (1986)
    PW,
    /// Paraguay  (1974)
    PY,
    /// Qatar  (1974)
    QA,
    /// Réunion  (1974)
    RE,
    /// Romania  (1974)
    RO,
    /// Serbia  (2006) Republic of Serbia
    RS,
    /// Russian Federation  (1992) ISO country name follows UN designation (common name: Russia); RU formerly reserved indeterminately for Burundi
    RU,
    /// Rwanda  (1974)
    RW,
    /// Saudi Arabia  (1974)
    SA,
    /// Solomon Islands  (1974) Code taken from former name: British Solomon Islands
    SB,
    /// Seychelles  (1974)
    SC,
    /// Sudan  (1974)
    SD,
    /// Sweden  (1974)
    SE,
    /// Singapore  (1974)
    SG,
    /// Saint Helena, Ascension and Tristan da Cunha  (1974) Previous ISO country name: Saint Helena.
    SH,
    /// Slovenia  (1992)
    SI,
    /// Svalbard and Jan Mayen  (1974) Previous ISO name: Svalbard and Jan Mayen Islands
    SJ,

    /// Slovakia  (1993) SK previously represented the Kingdom of Sikkim
    SK,
    /// Sierra Leone  (1974)
    SL,
    /// San Marino  (1974)
    SM,
    /// Senegal  (1974)
    SN,
    /// Somalia  (1974)
    SO,
    /// Suriname  (1974) Previous ISO country name: Surinam
    SR,
    /// South Sudan  (2011)
    SS,
    /// Sao Tome and Principe  (1974)
    ST,
    /// El Salvador  (1974)
    SV,
    /// Sint Maarten (Dutch part)  (2010) The French part of Saint Martin island is assigned code MF
    SX,
    /// Syrian Arab Republic  (1974) ISO country name follows UN designation (common name and previous ISO country name: Syria)
    SY,
    /// Eswatini  (1974) Previous ISO country name: Swaziland
    SZ,
    /// Turks and Caicos Islands  (1974)
    TC,
    /// Chad  (1974) Code taken from name in French: Tchad
    TD,
    /// French Southern Territories  (1979) Covers the French Southern and Antarctic Lands except Adélie Land
    TF,

    /// Togo  (1974)
    TG,
    /// Thailand  (1974)
    TH,
    /// Tajikistan  (1992)
    TJ,
    /// Tokelau  (1974) Previous ISO country name: Tokelau Islands
    TK,
    /// Timor-Leste  (2002) Name changed from East Timor (TP)
    TL,
    /// Turkmenistan  (1992)
    TM,
    /// Tunisia  (1974)
    TN,
    /// Tonga  (1974)
    TO,
    /// Türkiye  (1974) Previous ISO country name: Turkey
    TR,
    /// Trinidad and Tobago  (1974)
    TT,
    /// Tuvalu  (1977)
    TV,
    /// Taiwan, Province of China  (1974) Covers the current jurisdiction of the Republic of China
    TW,

    /// Tanzania, United Republic of  (1974)
    TZ,
    /// Ukraine  (1974) Previous ISO country name: Ukrainian SSR
    UA,

    /// Uganda  (1974)
    UG,
    /// United States Minor Outlying Islands  (1986) Consists of nine minor insular areas of the United States: Baker Island, Howland Island, Jarvis Island, Johnston Atoll, Kingman Reef, Midway Islands, Navassa Island, Palmyra Atoll, and Wake Island .um ccTLD was revoked in 2007[19]
    UM,

    /// United States of America  (1974) Previous ISO country name: United States
    US,
    /// Uruguay  (1974)
    UY,
    /// Uzbekistan  (1992)
    UZ,
    /// Holy See  (1974) Covers Vatican City, territory of the Holy See
    VA,

    /// Saint Vincent and the Grenadines  (1974)
    VC,
    /// Venezuela, Bolivarian Republic of  (1974) Previous ISO country name: Venezuela
    VE,
    /// Virgin Islands (British)  (1974)
    VG,
    /// Virgin Islands (U.S.)  (1974)
    VI,
    /// Viet Nam  (1974) ISO country name follows UN designation (common name: Vietnam)
    VN,

    /// Vanuatu  (1980) Name changed from New Hebrides (NH)
    VU,
    /// Wallis and Futuna  (1974) Previous ISO country name: Wallis and Futuna Islands
    WF,
    /// Samoa  (1974) Code taken from former name: Western Samoa
    WS,
    /// Yemen  (1974) Previous ISO country name: Yemen, Republic of (for three years after the unification)
    YE,

    /// Mayotte  (1993)
    YT,
    /// South Africa  (1974) Code taken from name in Dutch: Zuid-Afrika
    ZA,
    /// Zambia  (1974)
    ZM,
    /// Zimbabwe  (1980) Name changed from Southern Rhodesia (RH)
    ZW,
}

impl IntoSpotifyParam for Market {
    fn into_spotify_param(self) -> Option<String> {
        Some(self.to_string())
    }
}
