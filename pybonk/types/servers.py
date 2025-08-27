from typing import NamedTuple


class Server(NamedTuple):
    api_name: str
    latitude: float
    longitude: float
    country: str


Warsaw: Server = Server("b2warsaw1", 52.2370, 21.0175, "PL")
Paris: Server = Server("b2paris1", 48.8647, 2.3490, "FR")
Stockholm: Server = Server("b2stockholm1", 59.3346, 18.0632, "SE")
Frankfurt: Server = Server("b2frankfurt1", 50.1109, 8.6821, "GE")
Amsterdam: Server = Server("b2amsterdam1", 52.3779, 4.897, "NL")
London: Server = Server("b2london1", 51.5098, -0.1180, "UK")
Seoul: Server = Server("b2seoul1", 37.5326, 127.0246, "KR")
Seattle: Server = Server("b2seattle1", 47.6080, -122.3352, "US")
SanFrancisco: Server = Server("b2sanfrancisco1", 37.7740, -122.4312, "US")
Mississippi: Server = Server("b2river1", 35.5147, -89.9125, "US")
Dallas: Server = Server("b2dallas1", 32.7792, -96.8089, "US")
NewYork: Server = Server("b2ny1", 40.7306, -73.9352, "US")
Atlanta: Server = Server("b2atlanta1", 33.7537, -84.3863, "US")
Sydney: Server = Server("b2sydney1", -33.8651, 151.2099, "AU")
Brazil: Server = Server("b2brazil1", -22.9083, -43.1963, "BR")
