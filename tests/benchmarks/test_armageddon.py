import json
from pathlib import Path
from typing import List

import pytest

from pydantic_core import Schema, SchemaValidator

from .test_benchmarks import BaseModel, skip_pydantic

THIS_DIR = Path(__file__).parent
data_json = (THIS_DIR / 'armageddon.json').read_text()


@skip_pydantic
@pytest.mark.benchmark(group='armageddon')
def test_armageddon_pyd(benchmark):
    class MetadataDto(BaseModel):
        dataVersion: str
        matchId: str
        participants: List[str]

    class PerkStatsDto(BaseModel):
        defense: int
        flex: int
        offense: int

    class PerkStyleSelectionDto(BaseModel):
        perk: int
        var1: int
        var2: int
        var3: int

    class PerkStyleDto(BaseModel):
        description: str
        selections: List[PerkStyleSelectionDto]
        style: int

    class PerksDto(BaseModel):
        statPerks: PerkStatsDto
        styles: List[PerkStyleDto]

    class ParticipantDto(BaseModel):
        assists: int
        baronKills: int
        bountyLevel: int
        champExperience: int
        champLevel: int
        championId: int
        championName: str
        championTransform: int
        consumablesPurchased: int
        damageDealtToBuildings: int
        damageDealtToObjectives: int
        damageDealtToTurrets: int
        damageSelfMitigated: int
        deaths: int
        detectorWardsPlaced: int
        doubleKills: int
        dragonKills: int
        firstBloodAssist: bool
        firstBloodKill: bool
        firstTowerAssist: bool
        firstTowerKill: bool
        gameEndedInEarlySurrender: bool
        gameEndedInSurrender: bool
        goldEarned: int
        goldSpent: int
        individualPosition: str
        inhibitorKills: int
        inhibitorTakedowns: int
        inhibitorsLost: int
        item0: int
        item1: int
        item2: int
        item3: int
        item4: int
        item5: int
        item6: int
        itemsPurchased: int
        killingSprees: int
        kills: int
        lane: str
        largestCriticalStrike: int
        largestKillingSpree: int
        largestMultiKill: int
        longestTimeSpentLiving: int
        magicDamageDealt: int
        magicDamageDealtToChampions: int
        magicDamageTaken: int
        neutralMinionsKilled: int
        nexusKills: int
        nexusTakedowns: int
        nexusLost: int
        objectivesStolen: int
        objectivesStolenAssists: int
        participantId: int
        pentaKills: int
        perks: PerksDto
        physicalDamageDealt: int
        physicalDamageDealtToChampions: int
        physicalDamageTaken: int
        profileIcon: int
        puuid: str
        quadraKills: int
        riotIdName: str
        riotIdTagline: str
        role: str
        sightWardsBoughtInGame: int
        spell1Casts: int
        spell2Casts: int
        spell3Casts: int
        spell4Casts: int
        summoner1Casts: int
        summoner1Id: int
        summoner2Casts: int
        summoner2Id: int
        summonerId: str
        summonerLevel: int
        summonerName: str
        teamEarlySurrendered: bool
        teamId: int
        teamPosition: str
        timeCCingOthers: int
        timePlayed: int
        totalDamageDealt: int
        totalDamageDealtToChampions: int
        totalDamageShieldedOnTeammates: int
        totalDamageTaken: int
        totalHeal: int
        totalHealsOnTeammates: int
        totalMinionsKilled: int
        totalTimeCCDealt: int
        totalTimeSpentDead: int
        totalUnitsHealed: int
        tripleKills: int
        trueDamageDealt: int
        trueDamageDealtToChampions: int
        trueDamageTaken: int
        turretKills: int
        turretTakedowns: int
        turretsLost: int
        unrealKills: int
        visionScore: int
        visionWardsBoughtInGame: int
        wardsKilled: int
        wardsPlaced: int
        win: bool

    class BanDto(BaseModel):
        championId: int
        pickTurn: int

    class ObjectiveDto(BaseModel):
        first: bool
        kills: int

    class ObjectivesDto(BaseModel):
        baron: ObjectiveDto
        champion: ObjectiveDto
        dragon: ObjectiveDto
        inhibitor: ObjectiveDto
        riftHerald: ObjectiveDto
        tower: ObjectiveDto

    class TeamDto(BaseModel):
        bans: List[BanDto]
        objectives: ObjectivesDto
        teamId: int
        win: bool

    class InfoDto(BaseModel):
        gameCreation: int
        gameDuration: int
        gameEndTimestamp: int
        gameId: int
        gameMode: str
        gameName: str
        gameStartTimestamp: int
        gameType: str
        gameVersion: str
        mapId: int
        participants: List[ParticipantDto]
        platformId: str
        queueId: int
        teams: List[TeamDto]
        tournamentCode: str

    class MatchDto(BaseModel):
        metadata: MetadataDto
        info: InfoDto

    data = json.loads(data_json)
    benchmark(MatchDto.parse_obj, data)


@pytest.fixture(scope='module')
def core_validation_schema():
    class MetadataDto:
        __slots__ = '__dict__', '__fields_set__'

    metadata_dto: Schema = {
        'type': 'model-class',
        'class_type': MetadataDto,
        'model': {
            'type': 'model',
            'fields': {
                'dataVersion': {'type': 'str'},
                'matchId': {'type': 'str'},
                'participants': {'type': 'list', 'items': 'str'},
            },
        },
    }

    class PerkStatsDto:
        __slots__ = '__dict__', '__fields_set__'

    perk_stats_dto: Schema = {
        'type': 'model-class',
        'class_type': PerkStatsDto,
        'model': {
            'type': 'model',
            'fields': {'defense': {'type': 'int'}, 'flex': {'type': 'int'}, 'offense': {'type': 'int'}},
        },
    }

    class PerkStyleSelectionDto:
        __slots__ = '__dict__', '__fields_set__'

    perk_style_selection_dto: Schema = {
        'type': 'model-class',
        'class_type': PerkStyleSelectionDto,
        'model': {
            'type': 'model',
            'fields': {
                'perk': {'type': 'int'},
                'var1': {'type': 'int'},
                'var2': {'type': 'int'},
                'var3': {'type': 'int'},
            },
        },
    }

    class PerkStyleDto:
        __slots__ = '__dict__', '__fields_set__'

    perk_style_dto: Schema = {
        'type': 'model-class',
        'class_type': PerkStyleDto,
        'model': {
            'type': 'model',
            'fields': {
                'description': {'type': 'str'},
                'selections': {'type': 'list', 'items': perk_style_selection_dto},
                'style': {'type': 'int'},
            },
        },
    }

    class PerksDto:
        __slots__ = '__dict__', '__fields_set__'

    perks_dto: Schema = {
        'type': 'model-class',
        'class_type': PerksDto,
        'model': {
            'type': 'model',
            'fields': {'statPerks': perk_stats_dto, 'styles': {'type': 'list', 'items': perk_style_dto}},
        },
    }

    class ParticipantDto:
        __slots__ = '__dict__', '__fields_set__'

    participant_dto: Schema = {
        'type': 'model-class',
        'class_type': ParticipantDto,
        'model': {
            'type': 'model',
            'fields': {
                'assists': {'type': 'int'},
                'baronKills': {'type': 'int'},
                'bountyLevel': {'type': 'int'},
                'champExperience': {'type': 'int'},
                'champLevel': {'type': 'int'},
                'championId': {'type': 'int'},
                'championName': {'type': 'str'},
                'championTransform': {'type': 'int'},
                'consumablesPurchased': {'type': 'int'},
                'damageDealtToBuildings': {'type': 'int'},
                'damageDealtToObjectives': {'type': 'int'},
                'damageDealtToTurrets': {'type': 'int'},
                'damageSelfMitigated': {'type': 'int'},
                'deaths': {'type': 'int'},
                'detectorWardsPlaced': {'type': 'int'},
                'doubleKills': {'type': 'int'},
                'dragonKills': {'type': 'int'},
                'firstBloodAssist': {'type': 'bool'},
                'firstBloodKill': {'type': 'bool'},
                'firstTowerAssist': {'type': 'bool'},
                'firstTowerKill': {'type': 'bool'},
                'gameEndedInEarlySurrender': {'type': 'bool'},
                'gameEndedInSurrender': {'type': 'bool'},
                'goldEarned': {'type': 'int'},
                'goldSpent': {'type': 'int'},
                'individualPosition': {'type': 'str'},
                'inhibitorKills': {'type': 'int'},
                'inhibitorTakedowns': {'type': 'int'},
                'inhibitorsLost': {'type': 'int'},
                'item0': {'type': 'int'},
                'item1': {'type': 'int'},
                'item2': {'type': 'int'},
                'item3': {'type': 'int'},
                'item4': {'type': 'int'},
                'item5': {'type': 'int'},
                'item6': {'type': 'int'},
                'itemsPurchased': {'type': 'int'},
                'killingSprees': {'type': 'int'},
                'kills': {'type': 'int'},
                'lane': {'type': 'str'},
                'largestCriticalStrike': {'type': 'int'},
                'largestKillingSpree': {'type': 'int'},
                'largestMultiKill': {'type': 'int'},
                'longestTimeSpentLiving': {'type': 'int'},
                'magicDamageDealt': {'type': 'int'},
                'magicDamageDealtToChampions': {'type': 'int'},
                'magicDamageTaken': {'type': 'int'},
                'neutralMinionsKilled': {'type': 'int'},
                'nexusKills': {'type': 'int'},
                'nexusTakedowns': {'type': 'int'},
                'nexusLost': {'type': 'int'},
                'objectivesStolen': {'type': 'int'},
                'objectivesStolenAssists': {'type': 'int'},
                'participantId': {'type': 'int'},
                'pentaKills': {'type': 'int'},
                'perks': perks_dto,
                'physicalDamageDealt': {'type': 'int'},
                'physicalDamageDealtToChampions': {'type': 'int'},
                'physicalDamageTaken': {'type': 'int'},
                'profileIcon': {'type': 'int'},
                'puuid': {'type': 'str'},
                'quadraKills': {'type': 'int'},
                'riotIdName': {'type': 'str'},
                'riotIdTagline': {'type': 'str'},
                'role': {'type': 'str'},
                'sightWardsBoughtInGame': {'type': 'int'},
                'spell1Casts': {'type': 'int'},
                'spell2Casts': {'type': 'int'},
                'spell3Casts': {'type': 'int'},
                'spell4Casts': {'type': 'int'},
                'summoner1Casts': {'type': 'int'},
                'summoner1Id': {'type': 'int'},
                'summoner2Casts': {'type': 'int'},
                'summoner2Id': {'type': 'int'},
                'summonerId': {'type': 'str'},
                'summonerLevel': {'type': 'int'},
                'summonerName': {'type': 'str'},
                'teamEarlySurrendered': {'type': 'bool'},
                'teamId': {'type': 'int'},
                'teamPosition': {'type': 'str'},
                'timeCCingOthers': {'type': 'int'},
                'timePlayed': {'type': 'int'},
                'totalDamageDealt': {'type': 'int'},
                'totalDamageDealtToChampions': {'type': 'int'},
                'totalDamageShieldedOnTeammates': {'type': 'int'},
                'totalDamageTaken': {'type': 'int'},
                'totalHeal': {'type': 'int'},
                'totalHealsOnTeammates': {'type': 'int'},
                'totalMinionsKilled': {'type': 'int'},
                'totalTimeCCDealt': {'type': 'int'},
                'totalTimeSpentDead': {'type': 'int'},
                'totalUnitsHealed': {'type': 'int'},
                'tripleKills': {'type': 'int'},
                'trueDamageDealt': {'type': 'int'},
                'trueDamageDealtToChampions': {'type': 'int'},
                'trueDamageTaken': {'type': 'int'},
                'turretKills': {'type': 'int'},
                'turretTakedowns': {'type': 'int'},
                'turretsLost': {'type': 'int'},
                'unrealKills': {'type': 'int'},
                'visionScore': {'type': 'int'},
                'visionWardsBoughtInGame': {'type': 'int'},
                'wardsKilled': {'type': 'int'},
                'wardsPlaced': {'type': 'int'},
                'win': {'type': 'bool'},
            },
        },
    }

    class BanDto:
        __slots__ = '__dict__', '__fields_set__'

    ban_dto: Schema = {
        'type': 'model-class',
        'class_type': BanDto,
        'model': {'type': 'model', 'fields': {'championId': {'type': 'int'}, 'pickTurn': {'type': 'int'}}},
    }

    class ObjectiveDto:
        __slots__ = '__dict__', '__fields_set__'

    objective_dto: Schema = {
        'type': 'model-class',
        'class_type': ObjectiveDto,
        'model': {'type': 'model', 'fields': {'first': {'type': 'bool'}, 'kills': {'type': 'int'}}},
    }

    class ObjectivesDto:
        __slots__ = '__dict__', '__fields_set__'

    objectives_dto: Schema = {
        'type': 'model-class',
        'class_type': ObjectivesDto,
        'model': {
            'type': 'model',
            'fields': {
                'baron': objective_dto,
                'champion': objective_dto,
                'dragon': objective_dto,
                'inhibitor': objective_dto,
                'riftHerald': objective_dto,
                'tower': objective_dto,
            },
        },
    }

    class TeamDto:
        __slots__ = '__dict__', '__fields_set__'

    team_dto: Schema = {
        'type': 'model-class',
        'class_type': TeamDto,
        'model': {
            'type': 'model',
            'fields': {
                'bans': {'type': 'list', 'items': ban_dto},
                'objectives': objectives_dto,
                'teamId': {'type': 'int'},
                'win': {'type': 'bool'},
            },
        },
    }

    class InfoDto:
        __slots__ = '__dict__', '__fields_set__'

    info_dto: Schema = {
        'type': 'model-class',
        'class_type': InfoDto,
        'model': {
            'type': 'model',
            'fields': {
                'gameCreation': {'type': 'int'},
                'gameDuration': {'type': 'int'},
                'gameEndTimestamp': {'type': 'int'},
                'gameId': {'type': 'int'},
                'gameMode': {'type': 'str'},
                'gameName': {'type': 'str'},
                'gameStartTimestamp': {'type': 'int'},
                'gameType': {'type': 'str'},
                'gameVersion': {'type': 'str'},
                'mapId': {'type': 'int'},
                'participants': {'type': 'list', 'items': participant_dto},
                'platformId': {'type': 'str'},
                'queueId': {'type': 'int'},
                'teams': {'type': 'list', 'items': team_dto},
                'tournamentCode': {'type': 'str'},
            },
        },
    }

    class MatchDto:
        __slots__ = '__dict__', '__fields_set__'

    return SchemaValidator(
        {
            'type': 'model-class',
            'class_type': MatchDto,
            'model': {'type': 'model', 'fields': {'metadata': metadata_dto, 'info': info_dto}},
        }
    )


@pytest.mark.benchmark(group='armageddon')
def test_armageddon_core_python(benchmark, core_validation_schema):
    data = json.loads(data_json)
    benchmark(core_validation_schema.validate_python, data)


@pytest.mark.benchmark(group='armageddon')
def test_armageddon_core_json(benchmark, core_validation_schema):
    benchmark(core_validation_schema.validate_json, data_json)
