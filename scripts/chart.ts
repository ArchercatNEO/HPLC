import * as Chartist from 'chartist';

export interface ProcessedData 
{
    filteredData: number[][],
    times: number[],
    values: number[],
    areas : number[],
    baseline: number[],
    floor: number[],
    peaks: number[],
    gapped: (number | null)[],
    labels: string[],
    first: (number | null)[],
    second: (number | null)[]
}

export function GenerateChart(data: ProcessedData){
    const plots = {
        series: [
            {
                name: 'series-1',
                data: data.values,
            },
            {
                name: 'series-2',
                data: data.floor,
            },
            {
                name: 'series-3',
                data:  data.gapped
            },
            {
                name: "series-4",
                data: data.first
            },
            {
                name: "series-5",
                data: data.second
            }
        ],
    };

    const options = {
        fullWidth: true,
        height: "500px",

        series: {
            'series-1': {
                showLine: true,
                showPoint: false
            },
            'series-2': {
                showPoint: false
            },
            'series-3': {
                showLine: false,
                showPoint: true
            },
            'series-4': {
                showLine: false,
                showPoint: true
            },
            'series-5': {
                showLine: false,
                showPoint: true
            },
        }
    };


    /* Initialize the chart with the above settings */

    const the_chart = new Chartist.LineChart('#my-chart', plots, options);

    // Listening for draw events that get emitted by the Chartist chart
    the_chart.on('draw', ChangeDraw);

    type Listener = (
        Chartist.AxesDrawEvent |
        Chartist.PointDrawEvent |
        Chartist.LineDrawEvent |
        Chartist.AreaDrawEvent
    );
    function ChangeDraw(data: Listener)
    {
        // If the draw event was triggered from drawing a point on the line chart
        if(data.type === 'point') 
        {
            // We are creating a new path SVG element that draws a triangle around the point coordinates
            const triangle = new Chartist.Svg('path', 
            {
            d: ['M',
                data.x,
                data.y - 5,
                'L',
                data.x - 5,
                data.y + 3,
                'L',
                data.x + 5,
                data.y + 3,
                'z'].join(' '),
            style: 'fill-opacity: 1'
            }, 'ct-area');

            // With data.element we get the Chartist SVG wrapper and we can replace the original point drawn by Chartist with our newly created triangle
            data.element.replace(triangle);
        }
    }
}