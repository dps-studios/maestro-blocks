import type { MeasureContent } from '../types/blocks';

export function generateMeasureLilyPond(
  content: MeasureContent,
  clef: string = 'treble',
  timeSignature: string = '4/4'
): string {
  const [beats, noteValue] = timeSignature.split('/');
  
  let measureContent: string;
  
  switch (content.type) {
    case 'empty':
      measureContent = `s1*${beats}/${noteValue}`; // Spacer rests
      break;
    case 'chord':
      // Render chord symbol above staff
      measureContent = `
        \\override Score.ChordName.font-size = #2
        \\chordmode { ${content.symbol.toLowerCase()}1 }
        s1*${beats}/${noteValue}`;
      break;
    case 'notes':
      measureContent = content.notation;
      break;
    case 'rest':
      measureContent = `r1*${beats}/${noteValue}`;
      break;
  }

  return `
\\version "2.24.0"
\\paper {
  indent = 0\\mm
  line-width = 40\\mm
  oddHeaderMarkup = ""
  evenHeaderMarkup = ""
  oddFooterMarkup = ""
  evenFooterMarkup = ""
  top-margin = 2\\mm
  bottom-margin = 2\\mm
}
\\score {
  \\new Staff {
    \\clef ${clef}
    \\time ${timeSignature}
    \\override Staff.BarLine.transparent = ##t
    ${measureContent}
  }
  \\layout {
    \\context {
      \\Score
      \\remove "Bar_number_engraver"
    }
  }
}`;
}