import { GenerateChart, ProcessedData } from "./chart";

export function ProcessFile(event: Event, noise: number) {

    const file = event.target?.files[0];

    const reader = new FileReader();
    reader.readAsText(file, "UTF-8");
    reader.onload = (event) => Process(event, noise);
    
}

function Process(event: ProgressEvent<FileReader>, noise: number){

    const areas: number[] = [];
    const baseline: number[] = [0];
    const floor: number[] = [];
    const peaks: number[] = [];
    const gapped: number[] = [];
    
    //parse the file
    
    const raw = event.target?.result;

    if (raw === null || raw === undefined) {
        alert("File is empty or invalid");
        return;
    }

    const typedRaw = raw as string;
    const byRow = explode("\n", typedRaw);
    
    
    // Fill the data to be an array of arrays where each inner array is [time, value]
    const fullData = byRow.map(e => explode("\t", e)) as unknown as number[][];

    // Filter the times to only have data that happened between 8.85 and 36 time
    const filteredData = fullData.filter(e => 8.85 < e[0] && e[0] < 36);

    const times = filteredData.map(e => e[0]);
    const values = filteredData.map(e => e[1]);
    const labels = times.map(e => e % 150 === 0 ? e : "") as string[];

    
    /*************************************************************************************************
        esto calcula el baseline, al terminar 
         baseline[] tiene los indices donde hay que poner los puntos del baseline
    *************************************************************************************************/
    while (baseline[baseline.length - 1] + 1 < values.length){
        
        let gradient = Number.MAX_SAFE_INTEGER;
        let best = baseline[baseline.length - 1];
        // esto es para que no haya puntos consecutivos, 
        // lo forzamos a que sean al menos 10 puntos entre minimos
        const end = baseline[baseline.length - 1] + 5;
    
        for (let i = end + 1; i < values.length; i++)
        {
            let grad_i = (values[i] - values[end])/(i - end);
            if ( grad_i < gradient)
            {
                gradient = grad_i;
                best = i;
            }
        }
        
        baseline.push(best);
    }
    
    
    /*************************************************************************************************
        esto conecta todos los puntos del baseline interpolando los puntos anteriores
        y los guarda en $floor[]
    *************************************************************************************************/
    for (let i = 1; i < baseline.length; i++){
        
        // Left and right points 
        let left = baseline[i - 1];
        let right = baseline[i];
        
        let gradient = (values[right] - values[left])/(right - left);
    
        // Interpolate between the left and right points based on the gradient
        for (let x = 0; x < right - left; x++)
            floor.push(gradient * x + values[left]);   
    }
    
    
    for (let i = 1; i < values.length; i++){

        let area = 0;
    
        while (values[i - 1] > values[i]){
            area 
            += Math.abs((values[i - 1] - values[i])/2) 
            + values[i - 1] - floor[i - 1]
            + Math.abs((floor[i - 1] - floor[i])/2);
            i++;
        }
        
        if (i >= values.length) break;
    
        peaks.push(i);
        gapped[i] = values[i];
    
        while (values[i - 1] < values[i]){
            area 
            += Math.abs((values[i - 1] - values[i])/2) 
            + values[i - 1] - floor[i - 1]
            + Math.abs((floor[i - 1] - floor[i])/2);
            i++;
        }
            
        
        if (i >= values.length) break;
    
        areas.push(area);
        peaks.push(i);
        gapped[i] = values[i];
    }

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
    };
    console.log(data);

    GenerateChart(data);
}

    
function explode(breakCharacter: string, array: string){
    let output: string[] = [];
    let string: string = "";

    for (let i = 0; i < array.length; i++){
        if (array[i] !== breakCharacter)
            string += array[i];
        else {
            output.push(string);
            string = "";
        }
    }
    return output;
}