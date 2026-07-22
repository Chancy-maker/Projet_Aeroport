document.addEventListener("DOMContentLoaded", () => {
    createDestinationsChart();
    createOriginsChart();
    createDelaysChart();

    const refreshButton = document.getElementById("refresh-button");

    if (refreshButton) {
        refreshButton.addEventListener("click", () => {
            console.log("Actualisation des données demandée");

            /*
             * Le collègue pourra ensuite remplacer ceci par :
             *
             * fetch("/api/dashboard/summary")
             * fetch("/api/dashboard/top-destinations")
             * fetch("/api/dashboard/delays")
             */
        });
    }
});

function createDestinationsChart() {
    const element = document.getElementById("destinations-chart");

    if (!element) {
        return;
    }

    new Chart(element, {
        type: "bar",

        data: {
            labels: [
                "Chicago",
                "Atlanta",
                "Los Angeles",
                "Boston",
                "Orlando",
                "Charlotte",
                "San Francisco",
                "Miami",
                "Washington",
                "Fort Lauderdale"
            ],

            datasets: [
                {
                    label: "Nombre de vols",
                    data: [
                        17200,
                        16000,
                        14900,
                        14500,
                        13000,
                        12000,
                        11800,
                        11200,
                        10500,
                        9800
                    ],

                    backgroundColor: "#1d4ed8",
                    borderRadius: 7
                }
            ]
        },

        options: {
            responsive: true,
            maintainAspectRatio: false,

            indexAxis: "y",

            plugins: {
                legend: {
                    display: false
                }
            },

            scales: {
                x: {
                    beginAtZero: true
                }
            }
        }
    });
}

function createOriginsChart() {
    const element = document.getElementById("origins-chart");

    if (!element) {
        return;
    }

    new Chart(element, {
        type: "doughnut",

        data: {
            labels: ["JFK", "LGA", "EWR"],

            datasets: [
                {
                    data: [34, 31, 35],

                    backgroundColor: [
                        "#1d4ed8",
                        "#14213d",
                        "#60a5fa"
                    ]
                }
            ]
        },

        options: {
            responsive: true,
            maintainAspectRatio: false
        }
    });
}

function createDelaysChart() {
    const element = document.getElementById("delays-chart");

    if (!element) {
        return;
    }

    new Chart(element, {
        type: "bar",

        data: {
            labels: ["Départ", "Arrivée"],

            datasets: [
                {
                    label: "Retard moyen en minutes",
                    data: [12.6, 7.4],

                    backgroundColor: [
                        "#f59e0b",
                        "#1d4ed8"
                    ],

                    borderRadius: 8
                }
            ]
        },

        options: {
            responsive: true,
            maintainAspectRatio: false,

            plugins: {
                legend: {
                    display: false
                }
            },

            scales: {
                y: {
                    beginAtZero: true
                }
            }
        }
    });
}