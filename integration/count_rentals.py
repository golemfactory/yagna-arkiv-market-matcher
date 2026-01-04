import requests

URL = "http://127.0.0.1:36200/rentals"

def main():
    response = requests.get(URL)
    response.raise_for_status()  # fail fast on HTTP errors

    data = response.json()
    active_rentals = data.get("rentals", {}).get("activeRentals", [])

    print(f"Number of active rentals: {len(active_rentals)}")

if __name__ == "__main__":
    main()
