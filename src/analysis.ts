import { GenerateChart, ProcessedData } from "./chart";

export function Process(event: ProgressEvent<FileReader>, noise: number){

    const raw = event.target?.result as string | null | undefined;

    if (raw === null || raw === undefined) {
        alert("File is empty or invalid");
        return;
    }
    
    const byRow = raw.split("\r\n");
    
    // Fill the data to be an array of arrays where each inner array is [time, value]
    const fullData = byRow.map(e => e.split("\t"));
    const numberData = fullData.map(e => e.map(d => parseFloat(d)));

    // Filter the times to only have data that happened between 8.85 and 36 time
    const filteredData = numberData.filter(e => 8.85 < e[0] && e[0] < 36);

    const times = filteredData.map(e => e[0]);
    const values = filteredData.map(e => e[1]);
    const labels = times.map(e => e % 150 === 0 ? e : "") as string[];

    
    const baseline = GenerateBaseline(values);
    const floor = Interpolate(baseline, values);
    const {peaks, gapped, first0, second2, valleys, second}  = GeneratePeaks(values, noise);
    const {areas, ranges} = GenerateAreas(values, floor, valleys);

    const table: string[] = []
    
    for (let i = 0; i < areas.length; i++)
        table.push([ranges[i].map(e => times[e]).join("-"), areas[i], times[peaks[i]]].join(", "))

    document.getElementById("table")!.innerHTML = table.join("<br>");
    
    const data: ProcessedData = {
        filteredData: filteredData,
        times: times,
        values: values,
        areas : areas,
        baseline: baseline,
        floor: floor,
        peaks: peaks,
        gapped: gapped,
        labels: labels,
        first: first0,
        second: second2,
        more: second
    };

    GenerateChart(data);
}

/**
 * Use gradients to draw the tightest possible line that goes under every point
 * @param array the data
 * @returns 
 */
function GenerateBaseline(array: number[]): number[] {
    const output: number[] = [0];

    while (output[output.length - 1] + 1 < array.length){
        
        let gradient = Number.MAX_SAFE_INTEGER;
        let best = output[output.length - 1];
        // esto es para que no haya puntos consecutivos, 
        // lo forzamos a que sean al menos 10 puntos entre minimos
        const end = output[output.length - 1] + 5;
    
        for (let i = end + 1; i < array.length; i++)
        {
            let grad_i = (array[i] - array[end])/(i - end);
            if ( grad_i < gradient)
            {
                gradient = grad_i;
                best = i;
            }
        }
        
        output.push(best);
    }

    return output;
}

function GeneratePeaks(values: number[], noise: number) {
    
    const first = Derivative(values);
    const second = Derivative(first);

    console.log(first);

    const firstIndex: number[] = [];
    for (let i = 1; i < first.length; i++){
        if (first[i - 1] * first[i] <= -noise) firstIndex.push(i)
        console.log(first[i - 1] * first[i])
    }
    
    console.log(firstIndex)

    const second2: any[] = []/*second.map((e, i) => e < -noise ? values[i] : null);*/
    
    const secondIndex: number[] = [];
    for (let i = 1; i < second.length; i++)
        if (second[i - 1] * second[i] <= -noise) secondIndex.push(i)

    const first0: (null | number)[] = 
    Array(values.length).fill(null).map((_, i) => firstIndex.includes(i) ? values[i] : null);

    //const second0: (null | number)[] = 
    //Array(values.length).fill(null).map((_, i) => secondIndex.includes(i) ? values[i] : null);
    

    const peaks: number[] = [];
    const valleys: number[] = [];
    const gapped: number[] = [];

    for (let i = 1; i < values.length; i++){
        
        // While the graph is going down keep searching
        while (values[i - 1] > values[i]) i++;
        
        if (i >= values.length) break;
        
        // Found minimum
        valleys.push(i);
        gapped[i] = values[i];

        // While the graph is going up keep searching
        while (values[i - 1] < values[i]) i++
        
        if (i >= values.length) break;
        
        // Found maximum
        peaks.push(i);
        gapped[i] = values[i];
    }



    return {
        peaks,
        valleys,
        gapped,
        first0,
        second2,
        second
    };
}

function GenerateAreas(values: number[], floor: number[], peaks: number[]) {
    
    const ranges: number[][] = [];
    const output: number[] = [];
    for (let i = 1; i < peaks.length; i++){
        ranges.push([peaks[i - 1], peaks[i]])
        output.push(AreaBetween(values, floor, peaks[i - 1], peaks[i]))
    }
        
    return {
        ranges: ranges,
        areas : output
    };

}

function AreaBetween(values: number[], floor: number[], start: number, end: number) {
    let area = 0;
    for (let i = start; i < end; i++)
    {
        area += ( values[i] - floor[i] + values[i + 1] - floor[i + 1] ) / 2
    }
    return area;
}

function Derivative(array: number[]) {
    const output: number[] = [array[0], array[1]]
    for (let i = 2; i < array.length; i++)
        output.push((array[i] - array[i - 2])/2)
    return output;
}

/** 
 * Use Interpolation and gradients to draw a straight line between every point in indexArray using valuesArray 
*/
function Interpolate(indexArray: number[], valuesArray: number[]): number[] {
    const output: number[] = [];

    for (let i = 1; i < indexArray.length; i++){
        
        // Left and right points 
        let left = indexArray[i - 1];
        let right = indexArray[i];
        
        let gradient = (valuesArray[right] - valuesArray[left])/(right - left);
    
        // Interpolate between the left and right points based on the gradient
        for (let x = 0; x < right - left; x++)
            output.push(gradient * x + valuesArray[left]);   
    }

    return output;
}