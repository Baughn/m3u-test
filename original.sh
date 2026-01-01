#!/usr/bin/env bash
me=$(basename "$0")
export me

function usage(){
    cat >&2 <<ENDOFHELP
Usage: $me [-r|--relative] [-c|--children] [-h|--help] [-f|--force] TARGET [DESTINATION]

$me examines rom media files in TARGET and subdirs
and for each set of filenames that only differs from a order
identifier - (Disc #) and similar constructs - attempts to  
write a ordered m3u in DESTINATION with either absolute or
optionally with relative paths to those media files.
$me tolerates different disc games in the same dir,
but not for floppy games because of more hacks and label
information in a single floppies causes incomplete m3us.
Instead by default it creates a single m3u per dir if it
finds a floppy format in the directory.

If DESTINATION is missing, the m3u files will be created
on the directory the rom media files were found in, unless
-c is turned on.

    -h show this help
    -r m3us will have relative paths to the media files
    -c the TARGET will be treated as a source of top level
    directory names to be placed on DESTINATION with the
    m3us resulting from the search of each child directory
    created inside the new directories, you can use this
    to create shallow top-level copy of dirnames with m3us
    If no DESTINATION is given, a error stops processing
    If TARGET has no directories, no output is done
    TARGET and DESTINATION are not searched for games
    -f force the usage of the disc m3u strategy for floppies
ENDOFHELP
}
export RELATIVE=1
export CHILDREN=1
export FORCE=1
#silent mode, h or r or - followed by a 'argument' to handle long options 
#(notice that all of these require - already, including '-')
while getopts ":hrfc-:" opt; do
  case $opt in
    -)
        case "${OPTARG}" in
            relative) RELATIVE=0; ;;
            children) CHILDREN=0; ;;
            force) FORCE=0; ;;
            help) usage; exit 0; ;;
            *) usage; exit 1; ;;
        esac;;
    r) RELATIVE=0;  ;;
    c) CHILDREN=0;  ;;
    f) FORCE=0; ;;
    h) usage; exit 0; ;;
    *) usage; exit 1; ;;
  esac
done
shift $((OPTIND-1))
[[ "$#" -eq 0 ]]  && { usage; exit 1; }
[[ "$#" -gt 2 ]]  && { echo  >&2 "$me: $# positional arguments not allowed"; exit 1; }
[[ ! -d "$PWD" ]] && { echo  >&2 "$me: could not retrieve current directory"; exit 1; }
[[ -z "$2" ]] && [[ "$CHILDREN" -eq 0 ]] && { echo  >&2 "$me: the children switch requires a DESTINATION"; exit 1; }

#make sure both positional args (both dirs, that may not exist) tolerate '-'
#at the start of their name by absolutizing when necessary
TARGET=$(realpath -- "$1")
[[ -d "$TARGET" ]] || { echo >&2 "$me: target directory to search for m3u compatible files does not exist"; exit 1; }

#since this is going to be used on dir trees and delete m3u
#it's better to check if some random ancient dos or windows
#binary file format (i know of one in myst 3) is not named *.m3u
#and absolutely refuse to delete non text m3u. 
#I know there are cases where a normal m3u gets confused as a
#non text m3u and you have to delete it manually to progress,
#but I prefer that to accidental deletion
#arg1: m3u dir to check 
check_m3us(){
    #setup the global glob options bere because
    #this is done before any glob in program
    #the program calls find -exec with itself
    #which loses environment settings like shopt
    shopt -q -s nocasematch nocaseglob nullglob dotglob || true

    FAKEM3U=1
    for i in "$1"/*.m3u; do
        file -i -e 'csv' "$i" | grep -qE ': text/plain; charset' || { FAKEM3U=0; echo >&2 "$me: $i may not be a m3u text file."; }
    done
    if [[ "$FAKEM3U" -eq 0 ]]; then
        echo >&2 "$me: refusing to delete suspected non text m3u files from directory $1, delete manually and rerun to progress if you're sure"
        return 1
    fi
    return 0
}
export -f check_m3us

#function and array to turn strings containing english substrings one,two..., thirty-one or BOOT,A,B,...Z,SAVE,SYSTEM to numbers
disc_string_to_number(){
    #associative arrays can't be exported so dont even think of moving it out
    declare -A numerals_alpha_map=( [save]=99 [boot]=0 [thirty-one]=31 [thirty]=30 [twenty-nine]=29 [twenty-eight]=28 [twenty-seven]=27 [twenty-six]=26 [twenty-five]=25 [twenty-four]=24 [twenty-three]=23 [twenty-two]=22 [twenty-one]=21 [twenty]=20 [nineteen]=19 [eighteen]=18 [seventeen]=17 [sixteen]=16 [fifteen]=15 [fourteen]=14 [thirteen]=13 [twelve]=12 [eleven]=11 [ten]=10 [nine]=9 [eight]=8 [seven]=7 [six]=6 [five]=5 [four]=4 [three]=3 [two]=2 [one]=1 [zero]=0 [A]=1 [B]=2 [C]=3 [D]=4 [E]=5 [F]=6 [G]=7 [H]=8 [I]=9 [J]=10 [K]=11 [L]=12 [M]=13 [N]=14 [O]=15 [P]=16 [Q]=17 [R]=18 [S]=19 [T]=20 [U]=21 [V]=22 [W]=23 [X]=24 [Y]=25 [Z]=26 )
    local numerals_ordered=(boot save thirty-one thirty twenty-nine twenty-eight twenty-seven twenty-six twenty-five twenty-four twenty-three twenty-two twenty-one twenty nineteen eighteen seventeen sixteen fifteen fourteen thirteen twelve eleven ten nine eight seven six five four three two one zero)
    local alpha_ordered=(A  B  C  D  E  F  G  H  I  J  K  L  M  N  O  P  Q  R  S  T  U  V  W  X  Y  Z)
        
    for key in "${numerals_ordered[@]}"; do
        if [[ "$1" == "$key" ]]; then
            echo "${numerals_alpha_map[$key]}"
            return
        fi
    done
    
    shopt -u nocasematch || true #turn off (case on)
    for key in "${alpha_ordered[@]}"; do
        if [[ "$1" == "$key" ]]; then
            echo "${numerals_alpha_map[$key]}"
            shopt -s nocasematch || true
            return
        fi
    done
    shopt -s nocasematch || true #turn on (case off)

    echo ""
}
export -f disc_string_to_number

extract_number(){
    local num
    # Try to extract the first number in the string
    num=$(grep -oE '[0-9]+' <<<"$1" | head -n1)
    if [[ -n $num ]]; then
        #remove leading 0's (version sort dislikes them)
        echo $((10#$num))
        return 0
    fi
    #split into words, try to turn right words into a number
    read -r -a words <<<"$1"
    local len=${#words[@]}

    if (( len == 0 )); then
        echo ""
    elif (( len == 1 )); then
        #since the call echos thats enough to "return" the number
        disc_string_to_number "${words[0]}"
    else
        disc_string_to_number "${words[1]}"
    fi
}
export -f extract_number

#$1 is the filename without the path
#function writes to 3 associative arrays filename, fileshortname and filedisc
segmentname(){
    local name="${1%.*}"  #remove extension
    local disc
    #find the first occurrence of the order pattern; parenthesis not captured
    order_regex='\s*(floppy\s|diskette\s|disk\s|cd\s|disc\s|boot|save)[^)(]*'
    disc=$(echo -n "$name" | grep -oiP -e '(?<=\()'"$order_regex"'(?=\))' | head -n 1)
    disc=$(extract_number "$disc")
    if [[ -n "$disc" ]]; then
        #remove all occurrences of the pattern including parentheses
        name=$(echo "$name" | sed -E "s/\(${order_regex}\)//gi")
    else #no disc group effectively means only 1 for detection purpose later
        disc="1"
    fi
    #types of medium can have 2 sides, including some cds (dualdiscs), tapes, floppy
    side_regex='\s*side\s[^)(]*'
    side=$(echo -n "$name" | grep -oiP -e '(?<=\()'"$side_regex"'(?=\))' | head -n 1)
    side=$(extract_number "$side")    
    if [[ -n "$side" ]]; then
        name=$(echo "$name" | sed -E "s/\(${side_regex}\)//gi")
        side=".$side" #for version sort to sort inside sides if any too
    fi

    #removing the group(s) made a 'hole' which has two interesting cases, | (cd x) | and | (cd x).ext|
    #it's unlikely that multiple dumps of the same game will differ in the number of spaces and be in the same dir
    #so space manipulation so the final result looks good should be ok. Extension must be kept though

    name=$(echo "$name" | sed -Ee "s/(^\s+|\s+$)//g" ) #trim leading and trailing spaces
    name=$(echo "$name" | sed -Ee "s/\s\s*/ /g" )      #uniquify spaces
    #echo "$name | $disc$side"
    filename["$1"]="$name"
    filedisc["$1"]="${disc}${side}"
}
export -f segmentname

#$1 is the rom dir $2 is the m3u dir parent (which can be the current dir)
create(){
    cd -- "$1" || { echo >&2 "$me: inaccessible directory: $1"; return; }

    if [[ "$2" == "." ]]; then
        check_m3us "$PWD" || return
        rm -f ./*.m3u || { echo >&2 "$me: couldn't delete the previous m3u files"; return; }
    fi

    names=(*.{ipf,adf,adz,dms,dim,d64,d71,d81,d88,dsk,ima,fdi,qd,fds,tap,tzx,cas})
    FLOPPY=0
    if [[ "${#names[@]}" -eq 0 ]]; then
        FLOPPY=1
        #cue, toc, gdi and ccd are index files themselves so they have priority
        names=(*.{cue,toc,ccd,gdi})
        if [[ "${#names[@]}" -eq 0 ]]; then
            names=(*.{mds,cdi,img,iso,chd,rvz})
        fi
    fi    
    if [[ "${#names[@]}" -eq 0 ]]; then
        return 0
    fi
    #first create the associative arrays
    declare -A filename
    declare -A filedisc
    for n in "${names[@]}"; do
        segmentname "$n"
    done
    if [[ "$FORCE" -eq 0 ]]; then
        FLOPPY=1
    fi
    #sort uses V(ersion) sort for numerical sort, and the side sub sort
    if [[ "$FLOPPY" -eq 0 ]]; then
        #only sort the filedisc since we are assuming if its floppy its 1 game 1 dir
        mapfile -t sortednames < <(for key in "${names[@]}"; do
            printf '%s\n%s\0' "$key" "${filedisc[$key]}"
        done | sort -zVt$'\n' -k2,2 | cut -zd$'\n' -f1 | tr '\0' '\n' )
    else
        #disc last to give priority to rest of name
        mapfile -t sortednames < <(for key in "${names[@]}"; do
            printf '%s\n%s%s\0' "$key" "${filename[$key]}" "${filedisc[$key]}"
        done | sort -zVt$'\n' -k2,2 | cut -zd$'\n' -f1 | tr '\0' '\n' )
    fi
    ################################################################################################
    # M3U creation:
    # by default the program puts files in a dir in a new m3u if
    # the name minus extension and minus removed groups differs
    # this allows placing multiple games, versions or even compilations
    # with the same base title but a secondary title group in the same dir 
    # however some dumps (TOSEC but not only) can have random extras type groups
    # (boot|system|save|opening|scenario|data|...) besides the disk groups that
    # that are impossible to filter, so for floppies, if not optioned otherwise
    # put all files in the m3u
    while [[ "${#sortednames[@]}" -gt 0 ]] ; do

	    first="${sortednames[0]}"
	    fname="${filename[$first]}"
	    sortednames=("${sortednames[@]:1}") #removed the 1st element

	    gameset=("$first")
	    while [[ "${#sortednames[@]}" -gt 0 ]] ; do
	        second="${sortednames[0]}"
	        if [[ "$FLOPPY" -eq 0 || "$fname" == "${filename[$second]}" ]]; then
	            gameset+=("$second")
	            sortednames=("${sortednames[@]:1}")
	            continue
	        else
	            break
	        fi
	    done
	    m3u_name="${filename[$first]}.m3u"
	    if [[ "$RELATIVE" -eq 0 ]]; then
	        cuedir="$(realpath --relative-to "$2" "$1")/"
	        #ommit the cue dir if the same dir and relative
	        if [[ "$cuedir" == "./" ]]; then
		    cuedir=""
	        fi
	    else
	        cuedir="$1/"
	    fi
	    printf '%s\n' "${gameset[@]/#/$cuedir}" > "$2/$m3u_name"
    done
}
export -f create

createaux(){
        M3U_DIR="$(realpath -- "$2")"/"$(basename "$1")"
        mkdir -p "$M3U_DIR"       || { echo >&2 "$me: couldn't create the destination directory"; exit 1; }
        check_m3us "$M3U_DIR" || return
        rm    -f "$M3U_DIR"/*.m3u || { echo >&2 "$me: couldn't delete the previous m3u files"; exit 1; }
        LC_ALL=en_GB.UTF-8 find "$1" -type d -exec bash -c 'create "$1" "$2" ' -- "{}" "$M3U_DIR" \;
        #if completely empty, delete the dir since it didn't produce m3us
        if [ -z "$(ls "$M3U_DIR")" ]; then
            rm -r "$M3U_DIR"
        fi
}
export -f createaux



if [[ "$CHILDREN" -eq 0 ]]; then
    LC_ALL=en_GB.UTF-8 find "$TARGET" -mindepth 1 -maxdepth 1 -type d ! -path "$2" -exec bash -c 'createaux "$1" "$2" ' -- "{}" "$2" \;
else
    if [[ -z "$2" ]]; then
        #without a destination delete and write later on the rom dir (because we're saving m3u's on different dirs where we find games)
        M3U_DIR="."
    else
        #with a destination, delete now instead of later and absolutize
        M3U_DIR="$(realpath -- "$2")"
        mkdir -p "$M3U_DIR"       || { echo >&2 "$me: couldn't create the destination directory"; exit 1; }
        check_m3us "$M3U_DIR" || return
        rm    -f "$M3U_DIR"/*.m3u || { echo >&2 "$me: couldn't delete the previous m3u files"; exit 1; }
    fi
    #use utf-8 for this
    LC_ALL=en_GB.UTF-8 find "$TARGET" -type d -exec bash -c 'create "$1" "$2" ' -- "{}" "$M3U_DIR" \;
fi

