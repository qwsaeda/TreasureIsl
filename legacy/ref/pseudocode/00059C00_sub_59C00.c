/* ea:   0x00059C00 */
/* name: sub_59C00 */
/* size: 4056 */
unsigned int __fastcall sub_59C00(unsigned int *a1)
{
  unsigned int *v1; // r5
  unsigned int v2; // r1
  unsigned int result; // r0
  int v4; // r6
  unsigned int v5; // r1
  int v6; // r4
  int v7; // lr
  int v8; // r3
  int v9; // r2
  int v10; // r0
  unsigned int v11; // r9
  _DWORD *v12; // r10
  unsigned int v13; // r3
  int v14; // r4
  int v15; // r2
  int v16; // r1
  unsigned int v17; // r0
  unsigned int v18; // r6
  unsigned int v19; // r3
  unsigned int v20; // r5
  int v21; // r7
  int v22; // r8
  int v23; // r1
  int v24; // r1
  int v25; // r2
  int v26; // r3
  unsigned int v27; // r1
  unsigned int v28; // r4
  unsigned int v29; // r2
  unsigned int v30; // r3
  _BYTE *v31; // r4
  char v32; // r3
  int v33; // r1
  int v34; // r0
  bool v35; // nf
  int v36; // r1
  unsigned int v37; // r2
  int v38; // r1
  int v39; // r1
  int v40; // r2
  int v41; // r0
  int v42; // r2
  unsigned int v43; // r1
  int v44; // r2
  unsigned int v45; // r1
  _DWORD *v46; // r6
  int i; // r7
  int v48; // r5
  int v49; // r0
  int v50; // r0
  int v51; // r2
  int v52; // r1
  int v53; // r1
  unsigned int v54; // r1
  unsigned int v55; // r3
  int v56; // r2
  unsigned int v57; // r1
  unsigned int v58; // r1
  int v59; // r2
  unsigned int v60; // r4
  unsigned int v61; // r2
  int v62; // r0
  int v63; // r3
  bool v64; // cf
  int v65; // r0
  int v66; // r0
  int v67; // r1
  int v68; // r0
  _DWORD *v69; // r4
  int v70; // r3
  unsigned int v71; // r2
  unsigned int v72; // r5
  int v73; // r2
  int v74; // r4
  int v75; // r2
  unsigned int v76; // r3
  _DWORD *v77; // r4
  int v78; // r3
  unsigned int v79; // r2
  unsigned int v80; // r5
  int v81; // r2
  int v82; // r4
  int v83; // r2
  unsigned int v84; // r3
  _DWORD *v85; // r4
  int v86; // r3
  unsigned int v87; // r2
  unsigned int v88; // r5
  int v89; // r2
  int v90; // r4
  int v91; // r2
  unsigned int v92; // r3
  _DWORD *v93; // r4
  int v94; // r3
  unsigned int v95; // r2
  unsigned int v96; // r5
  int v97; // r2
  int v98; // r4
  int v99; // r2
  unsigned int v100; // r3
  int v101; // r9
  int v102; // r10
  unsigned int v103; // r8
  int v104; // r7
  int v105; // r6
  int v106; // r0
  unsigned int v107; // r1
  unsigned int v108; // r0
  unsigned int v109; // r5
  int v110; // r0
  int v111; // r4
  int v112; // r3
  int v113; // r2
  unsigned int v114; // r3
  int v115; // r1
  int v116; // r1
  unsigned int v117; // r3
  int v118; // r1
  _WORD *v119; // r0
  _WORD *v120; // r0
  int v121; // r1
  bool v122; // cc
  int v123; // r1
  bool v124; // cc
  int v126; // [sp+4h] [bp-38h]
  int v127; // [sp+8h] [bp-34h]
  unsigned int v128; // [sp+Ch] [bp-30h]
  unsigned int v129; // [sp+10h] [bp-2Ch]
  int v130; // [sp+14h] [bp-28h]

  v1 = a1;
  v2 = a1[25];
  v129 = 0;
  result = a1[30];
  if ( v2 >> result == 0 )
    return result;
LABEL_2:
  v127 = 0;
  v4 = 0;
  v5 = 0;
  v126 = 0;
  v128 = 0;
  if ( v1[19] == 0 )
    goto LABEL_180;
LABEL_3:
  v6 = v1[20] + 1304 * v5;
  v130 = v6;
  if ( *(_BYTE *)(v6 + 22) == 1 || *(_BYTE *)(v6 + 21) == 1 )
    goto LABEL_179;
  v7 = v1[20] + 1304 * v5;
  if ( *(_BYTE *)(v6 + 32) != 0 )
  {
    *(_DWORD *)(v6 + 64) = ((__int64)((unsigned __int64)v1[18] << 16)
                          / (int)((*(_DWORD *)(v6 + 28) << 8) / 0xAC44u * *(_DWORD *)(v6 + 1300))) >> 8;
    *(_BYTE *)(v6 + 32) = 0;
  }
  v8 = *(_DWORD *)(v6 + 72);
  v9 = *(_DWORD *)(v6 + 76);
  if ( v9 != v8 )
  {
    v45 = *(_DWORD *)(v6 + 68);
    if ( v45 < *(_DWORD *)(v6 + 64) )
    {
      *(_DWORD *)(v6 + 68) = v45 + 1;
    }
    else
    {
      *(_DWORD *)(v6 + 68) = 0;
      *(_DWORD *)(v6 + 76) = v9 + 1;
      if ( v9 + 1 != v8 )
      {
        v46 = (_DWORD *)v6;
        for ( i = 4; i != 0; --i )
        {
          v48 = v46[44];
          switch ( v48 )
          {
            case 0:
              if ( v46[56] != 0 )
              {
                v49 = v46[276];
                v46[276] = v49 + 1;
                if ( v49 == 2 )
                  v46[276] = 0;
                v50 = v46[276];
                if ( v50 != 0 )
                {
                  if ( v50 == 1 )
                  {
                    v46[32] = v46[284];
                  }
                  else if ( v50 == 2 )
                  {
                    v46[32] = v46[288];
                  }
                }
                else
                {
                  v46[32] = v46[280];
                }
              }
              break;
            case 1:
              v51 = v46[264] - v46[56];
              v46[264] = v51;
              v52 = *(_DWORD *)(a1[23] + 4 * v51);
              if ( v52 != 0 )
                v46[32] = v52;
              goto LABEL_102;
            case 2:
              v51 = v46[264] + v46[56];
              v46[264] = v51;
              v53 = *(_DWORD *)(a1[23] + 4 * v51);
              if ( v53 != 0 )
              {
                v46[32] = v53;
                v46[24] = v51;
              }
              else
              {
LABEL_102:
                v46[24] = v51;
              }
              break;
            case 3:
            case 5:
              v54 = v46[24];
              v55 = v46[268];
              if ( v54 > v55 )
              {
                v56 = v54 - v46[272];
                v46[24] = v56;
                if ( v56 < v55 || v56 < 0 )
                  v46[24] = v55;
              }
              v57 = v46[24];
              if ( v57 < v55 )
              {
                v58 = v46[272] + v57;
                v46[24] = v58;
                if ( v58 > v55 )
                  v46[24] = v55;
              }
              v46[32] = *(_DWORD *)(a1[23] + 4 * v46[24]);
              if ( v48 == 5 )
                goto LABEL_113;
              break;
            case 4:
            case 6:
              v60 = v46[292];
              if ( v60 < 0x20 )
                v61 = v46[24] + ((unsigned int)(dword_7BA70[v60] * v46[300]) >> 7);
              else
                v61 = v46[24] - ((unsigned int)(dword_7BA70[v60 - 32] * v46[300]) >> 7);
              v62 = v46[296];
              v63 = *(_DWORD *)(a1[23] + 4 * v61);
              v64 = v62 + v60 >= 0x40;
              v46[292] = v62 + v60;
              if ( v62 + v60 >= 0x40 )
                v62 = v62 + v60 - 64;
              v46[32] = v63;
              if ( v64 )
                v46[292] = v62;
              if ( v48 != 6 )
                break;
              v59 = v46[260] + v46[36];
              v46[36] = v59;
              if ( v59 >= 0 )
                goto LABEL_115;
              v46[36] = 0;
              v46[40] = 0;
              break;
            case 10:
LABEL_113:
              v59 = v46[260] + v46[36];
              v46[36] = v59;
              if ( v59 >= 0 )
              {
LABEL_115:
                if ( v59 > 64 )
                  v46[36] = 64;
                v46[40] = (unsigned int)(*(_DWORD *)(v130 + 12) * v46[36]) >> 6;
              }
              else
              {
                v46[36] = 0;
                v46[40] = 0;
              }
              break;
            case 14:
              v65 = v46[48];
              if ( v65 == 12 )
              {
                v68 = v46[312];
                if ( v68 != 0 )
                  v46[312] = v68 - 1;
                if ( v46[312] == 0 )
                  v46[40] = 0;
              }
              else if ( v65 == 13 )
              {
                v66 = v46[304];
                if ( v66 == -1 )
                {
                  v67 = v46[308];
                  v46[304] = 0xFFFFFF;
                  v46[24] = v67;
                  v46[32] = *(_DWORD *)(a1[23] + 4 * v67);
                }
                else if ( v66 != 0xFFFFFF )
                {
                  v46[304] = v66 - 1;
                }
              }
              break;
            default:
              break;
          }
          ++v46;
        }
      }
    }
    v4 = v126;
    goto LABEL_142;
  }
  v10 = *(_DWORD *)(v6 + 1272);
  if ( v10 != 0 )
  {
    *(_DWORD *)(v6 + 76) = 0;
    *(_DWORD *)(v6 + 1272) = v10 - 1;
    goto LABEL_142;
  }
  v11 = 0;
  v12 = (_DWORD *)v6;
  while ( 2 )
  {
    v13 = *(_DWORD *)(*(_DWORD *)(v7 + 44) + 4 * (*(_DWORD *)(v7 + 52) + (*(_DWORD *)(v7 + 48) << 8) + v11));
    v14 = v13 & 0xF0 | (v13 << 8 >> 28);
    v15 = BYTE1(v13);
    v16 = v13 & 0xF;
    v17 = v13 << 12;
    v18 = v13 >> 28;
    v19 = HIBYTE(v13);
    v12[48] = v18;
    v20 = v17 >> 28;
    v12[56] = v19;
    v21 = v19 & 0xF;
    v12[44] = v17 >> 28;
    v22 = v15 | (v16 << 8);
    v12[52] = v21;
    if ( v14 != 0 )
    {
      v12[28] = v14;
      v23 = *(_DWORD *)(v7 + 4 * v14 + 656);
      v12[36] = v23;
      v12[40] = (unsigned int)(*(_DWORD *)(v7 + 12) * v23) >> 6;
    }
    if ( v22 != 0 && v20 != 3 && v20 != 5 && v18 + 16 * v20 != 237 )
    {
      v12[60] = 0;
      v24 = dword_74F50[v22] + 84 * *(_DWORD *)(v7 + 4 * v12[28] + 784);
      v12[20] = v24;
      v12[24] = dword_73A50[v24];
    }
    v25 = v12[28];
    if ( v25 != 0 )
      v12[32] = *(_DWORD *)(a1[23] + 4 * v12[24]);
    switch ( v20 )
    {
      case 0u:
        if ( v19 != 0 )
        {
          v26 = v12[20];
          v12[276] = 0;
          v12[280] = v12[32];
          v12[284] = *(_DWORD *)(a1[23] + 4 * dword_73A50[v26 + v18]);
          v12[288] = *(_DWORD *)(a1[23] + 4 * dword_73A50[v26 + v21]);
        }
        goto LABEL_74;
      case 1u:
      case 2u:
        v12[264] = v12[24];
        goto LABEL_74;
      case 3u:
        if ( v22 != 0 )
        {
          v12[268] = dword_73A50[84 * *(_DWORD *)(v7 + 4 * v25 + 784) + dword_74F50[v22]];
        }
        else if ( v12[268] == 0 )
        {
          v12[268] = dword_73A50[v12[20]];
        }
        if ( v19 != 0 )
          v12[272] = v19;
        goto LABEL_74;
      case 4u:
        if ( v19 != 0 )
        {
          v12[296] = v18;
          v12[292] = 0;
          v12[300] = v21;
        }
        goto LABEL_74;
      case 5u:
      case 6u:
      case 0xAu:
        if ( v19 != 0 )
        {
          if ( v18 != 0 )
            v12[260] = v18;
          else
            v12[260] = -v21;
        }
        else
        {
          v12[260] = 0;
        }
        goto LABEL_74;
      case 9u:
        if ( v19 != 0 )
          v12[321] = (v21 + 16 * v18) << 24;
        v12[60] = v12[321];
        goto LABEL_74;
      case 0xBu:
        if ( *(_BYTE *)(v7 + 20) != 0 || v19 > *(_DWORD *)(v7 + 56) )
        {
          *(_DWORD *)(v7 + 1264) = v19;
          *(_DWORD *)(v7 + 1268) = 0;
        }
        else
        {
          *(_DWORD *)(v7 + 1264) = *(unsigned __int8 *)(*(_DWORD *)(v7 + 40) + 950);
        }
        goto LABEL_74;
      case 0xCu:
        v12[36] = v19;
        v12[40] = (*(_DWORD *)(v7 + 12) * v19) >> 6;
        goto LABEL_74;
      case 0xDu:
        v29 = *(unsigned __int8 *)(*(_DWORD *)(v7 + 40) + 950);
        v30 = *(_DWORD *)(v7 + 56) + 1;
        if ( v30 < v29 )
        {
          *(_DWORD *)(v7 + 1264) = v30;
          *(_DWORD *)(v7 + 1268) = 4 * (v12[52] + 10 * v12[48]);
        }
        else if ( *(_BYTE *)(v7 + 20) != 0 )
        {
          *(_DWORD *)(v7 + 1264) = *(_DWORD *)(v7 + 60);
          *(_DWORD *)(v7 + 1268) = 4 * (v12[52] + 10 * v12[48]);
        }
        else
        {
          *(_DWORD *)(v7 + 1264) = v29;
        }
        goto LABEL_74;
      case 0xEu:
        switch ( v18 )
        {
          case 6u:
            v31 = (_BYTE *)(v11 + v7 + 1276);
            v32 = *v31 & 0xF0;
            if ( v32 != 0 )
            {
              if ( v21 == 0 )
                break;
              v21 = (*v31 & 0xF) - 1;
            }
            else
            {
              if ( v21 == 0 )
              {
                *(_BYTE *)(v11 + v7 + 1280) = *(_DWORD *)(v7 + 52) >> 2;
                break;
              }
              v32 = -16;
            }
            if ( v21 != 0 )
            {
              *(_DWORD *)(v7 + 1264) = *(_DWORD *)(v7 + 56);
              *(_DWORD *)(v7 + 1268) = 4 * *(unsigned __int8 *)(v11 + v7 + 1280);
              *v31 = v21 | v32;
            }
            else
            {
              *v31 = 0;
            }
            break;
          case 0xAu:
            v33 = v12[36] + v21;
            v12[36] = v33;
            if ( v33 > 64 )
              v12[36] = 64;
            v12[40] = (unsigned int)(v12[36] * *(_DWORD *)(v7 + 12)) >> 6;
            break;
          case 0xBu:
            v34 = v12[36];
            v36 = v34 - v21;
            v35 = v34 - v21 < 0;
            if ( v34 - v21 < 0 )
              v34 = 0;
            v12[36] = v36;
            if ( v35 )
              v12[36] = v34;
            v12[40] = (unsigned int)(v12[36] * *(_DWORD *)(v7 + 12)) >> 6;
            break;
          case 0xCu:
            v12[312] = v21;
            break;
          case 0xDu:
            v12[308] = v22;
            v12[304] = v21 - 1;
            break;
          case 0xEu:
            *(_DWORD *)(v7 + 1272) = v21;
            break;
          default:
            break;
        }
LABEL_74:
        ++v11;
        ++v12;
        if ( v11 < 4 )
          continue;
        v37 = *(_DWORD *)(v7 + 1264);
        if ( v37 == 0xFFFFFF )
        {
          v41 = *(_DWORD *)(v7 + 52);
          *(_DWORD *)(v7 + 52) = v41 + 4;
          if ( v41 != 252 )
            goto LABEL_79;
          v42 = *(_DWORD *)(v7 + 40);
          v43 = *(_DWORD *)(v7 + 56) + 1;
          *(_DWORD *)(v7 + 52) = 0;
          *(_DWORD *)(v7 + 56) = v43;
          if ( v43 >= *(unsigned __int8 *)(v42 + 950) )
          {
            if ( *(_BYTE *)(v7 + 20) == 0 )
            {
              sub_5AF80(v7);
              v5 = v128;
              v1 = a1;
              v4 = v126;
              goto LABEL_179;
            }
            *(_DWORD *)(v7 + 56) = *(_DWORD *)(v7 + 60);
          }
          v4 = v126;
          v44 = *(unsigned __int8 *)(*(_DWORD *)(v7 + 56) + v42 + 952);
          *(_DWORD *)(v7 + 76) = 0;
          *(_DWORD *)(v7 + 48) = v44;
          goto LABEL_142;
        }
        v38 = *(_DWORD *)(v7 + 40);
        if ( v37 >= *(unsigned __int8 *)(v38 + 950) )
        {
          sub_5AF80(v7);
          v4 = v126;
          goto LABEL_178;
        }
        *(_DWORD *)(v7 + 56) = v37;
        v39 = *(unsigned __int8 *)(v37 + v38 + 952);
        v40 = *(_DWORD *)(v7 + 1268);
        *(_DWORD *)(v7 + 48) = v39;
        *(_DWORD *)(v7 + 52) = v40;
        *(_DWORD *)(v7 + 1264) = 0xFFFFFF;
LABEL_79:
        v4 = v126;
        *(_DWORD *)(v7 + 76) = 0;
LABEL_142:
        v69 = (_DWORD *)(v7 + 4 * *(_DWORD *)(v7 + 112));
        if ( v69[228] != 0 )
        {
          v70 = v69[132];
          if ( v70 != 0 )
          {
            v71 = *(_DWORD *)(v7 + 240);
            if ( v71 >= v70 + v69[100] )
              *(_DWORD *)(v7 + 240) = v71 - v70;
          }
          v72 = *(_DWORD *)(v7 + 240);
          if ( v72 < v69[68] )
          {
            v73 = *(char *)(v69[228] + HIWORD(v72));
            if ( a1[29] != 0 )
            {
              v74 = 2 * v73;
              v75 = *(_DWORD *)(v7 + 160);
              v76 = (unsigned int)(*(_DWORD *)(v7 + 256) * v75) >> 8;
              v4 += (v75 - v76) * v74;
              v127 += v76 * v74;
            }
            else
            {
              v4 += *(_DWORD *)(v7 + 160) * v73;
            }
            v126 = v4;
            *(_DWORD *)(v7 + 240) = *(_DWORD *)(v7 + 128) + v72;
          }
        }
        v77 = (_DWORD *)(v7 + 4 * *(_DWORD *)(v7 + 116));
        if ( v77[228] != 0 )
        {
          v78 = v77[132];
          if ( v78 != 0 )
          {
            v79 = *(_DWORD *)(v7 + 244);
            if ( v79 >= v78 + v77[100] )
              *(_DWORD *)(v7 + 244) = v79 - v78;
          }
          v80 = *(_DWORD *)(v7 + 244);
          if ( v80 < v77[68] )
          {
            v81 = *(char *)(v77[228] + HIWORD(v80));
            if ( a1[29] != 0 )
            {
              v82 = 2 * v81;
              v83 = *(_DWORD *)(v7 + 164);
              v84 = (unsigned int)(*(_DWORD *)(v7 + 260) * v83) >> 8;
              v4 += (v83 - v84) * v82;
              v127 += v84 * v82;
            }
            else
            {
              v4 += *(_DWORD *)(v7 + 164) * v81;
            }
            v126 = v4;
            *(_DWORD *)(v7 + 244) = *(_DWORD *)(v7 + 132) + v80;
          }
        }
        v85 = (_DWORD *)(v7 + 4 * *(_DWORD *)(v7 + 120));
        if ( v85[228] != 0 )
        {
          v86 = v85[132];
          if ( v86 != 0 )
          {
            v87 = *(_DWORD *)(v7 + 248);
            if ( v87 >= v86 + v85[100] )
              *(_DWORD *)(v7 + 248) = v87 - v86;
          }
          v88 = *(_DWORD *)(v7 + 248);
          if ( v88 < v85[68] )
          {
            v89 = *(char *)(v85[228] + HIWORD(v88));
            if ( a1[29] != 0 )
            {
              v90 = 2 * v89;
              v91 = *(_DWORD *)(v7 + 168);
              v92 = (unsigned int)(*(_DWORD *)(v7 + 264) * v91) >> 8;
              v4 += (v91 - v92) * v90;
              v127 += v92 * v90;
            }
            else
            {
              v4 += *(_DWORD *)(v7 + 168) * v89;
            }
            v126 = v4;
            *(_DWORD *)(v7 + 248) = *(_DWORD *)(v7 + 136) + v88;
          }
        }
        v93 = (_DWORD *)(v7 + 4 * *(_DWORD *)(v7 + 124));
        if ( v93[228] != 0 )
        {
          v94 = v93[132];
          if ( v94 != 0 )
          {
            v95 = *(_DWORD *)(v7 + 252);
            if ( v95 >= v94 + v93[100] )
              *(_DWORD *)(v7 + 252) = v95 - v94;
          }
          v96 = *(_DWORD *)(v7 + 252);
          if ( v96 < v93[68] )
          {
            v97 = *(char *)(v93[228] + HIWORD(v96));
            if ( a1[29] != 0 )
            {
              v98 = 2 * v97;
              v99 = *(_DWORD *)(v7 + 172);
              v100 = (unsigned int)(*(_DWORD *)(v7 + 268) * v99) >> 8;
              v4 += (v99 - v100) * v98;
              v127 += v100 * v98;
            }
            else
            {
              v4 += *(_DWORD *)(v7 + 172) * v97;
            }
            v126 = v4;
            *(_DWORD *)(v7 + 252) = *(_DWORD *)(v7 + 140) + v96;
          }
        }
LABEL_178:
        v5 = v128;
        v1 = a1;
LABEL_179:
        v128 = ++v5;
        if ( v5 < v1[19] )
          goto LABEL_3;
LABEL_180:
        v101 = 0;
        v102 = 0;
        v103 = 0;
        if ( v1[21] != 0 )
        {
          v104 = 0;
          do
          {
            v105 = v104 + v1[22];
            if ( *(_BYTE *)(v105 + 22) != 1 && *(_BYTE *)(v105 + 21) != 1 )
            {
              if ( *(_BYTE *)(v105 + 32) != 0 )
              {
                v106 = (*(_DWORD *)(v105 + 28) << 16) / (v1[18] << 8);
                *(_BYTE *)(v105 + 32) = 0;
                *(_DWORD *)(v105 + 44) = v106;
              }
              if ( *(_BYTE *)(v105 + 20) != 0 )
              {
                v107 = *(_DWORD *)(v105 + 48);
                v108 = *(_DWORD *)(v105 + 52);
                if ( v107 >= v108 )
                  *(_DWORD *)(v105 + 48) = v107 - v108;
              }
              v109 = *(_DWORD *)(v105 + 48);
              if ( v109 >= *(_DWORD *)(v105 + 52) )
              {
                sub_5AF80(v105);
              }
              else
              {
                v110 = *(_DWORD *)(v105 + 40);
                if ( *(_BYTE *)(v105 + 56) != 0 )
                  v111 = (*(unsigned __int8 *)(v110 + (v109 >> 8)) - 128) << 8;
                else
                  v111 = *(__int16 *)(((v109 >> 7) & 0x1FFFFFE) + v110);
                if ( a1[29] != 0 )
                {
                  v112 = *(_DWORD *)(v105 + 16);
                  if ( v112 == 128 )
                  {
                    v115 = v111 * *(_DWORD *)(v105 + 12);
                    v101 += v115;
                    v102 += v115;
                  }
                  else
                  {
                    v113 = 2 * *(_DWORD *)(v105 + 12);
                    v114 = (unsigned int)(v112 * v113) >> 8;
                    v101 += (v113 - v114) * v111;
                    v102 += v114 * v111;
                  }
                  *(_DWORD *)(v105 + 48) = v109 + *(_DWORD *)(v105 + 44);
                }
                else
                {
                  v101 += v111 * *(_DWORD *)(v105 + 12);
                  *(_DWORD *)(v105 + 48) = v109 + *(_DWORD *)(v105 + 44);
                }
              }
              v1 = a1;
            }
            ++v103;
            v104 += 60;
          }
          while ( v103 < v1[21] );
          v4 = v126;
        }
        if ( v1[30] != 0 )
        {
          v116 = (v101 >> 6) + 2 * v4;
          if ( v116 <= 0x7FFF )
          {
            if ( v116 < -32768 )
              LOWORD(v116) = 0x8000;
          }
          else
          {
            LOWORD(v116) = 0x7FFF;
          }
          *(_WORD *)(v1[28] + 2 * v129) = v116;
          v117 = ++v129;
          if ( v1[29] != 0 )
          {
            v118 = (v102 >> 6) + 2 * v127;
            if ( v118 <= 0x7FFF )
            {
              if ( v118 < -32768 )
                LOWORD(v118) = 0x8000;
              v120 = (_WORD *)(v1[28] + 2 * v117++);
              *v120 = v118;
              v129 = v117;
            }
            else
            {
              v119 = (_WORD *)(v1[28] + 2 * v117++);
              *v119 = 0x7FFF;
              v129 = v117;
            }
          }
        }
        else
        {
          v121 = (v4 + (v101 >> 7)) >> 7;
          v122 = v121 < 127;
          if ( v121 <= 127 )
            v122 = v121 < -128;
          else
            LOBYTE(v121) = 127;
          if ( v122 )
            LOBYTE(v121) = 0x80;
          *(_BYTE *)(v1[28] + v129) = v121 + 0x80;
          v117 = ++v129;
          if ( v1[29] != 0 )
          {
            v123 = (v127 + (v102 >> 7)) >> 7;
            v124 = v123 < 127;
            if ( v123 <= 127 )
              v124 = v123 < -128;
            else
              LOBYTE(v123) = 127;
            if ( v124 )
              LOBYTE(v123) = 0x80;
            *(_BYTE *)(v1[28] + v117++) = v123 + 0x80;
            v129 = v117;
          }
        }
        result = v1[30];
        if ( v117 < v1[25] >> result )
          goto LABEL_2;
        return result;
      case 0xFu:
        if ( v19 > 0x20 )
        {
          v27 = *(_DWORD *)(v7 + 28) << 8;
          v28 = 2 * v19 / 5;
          *(_DWORD *)(v7 + 1300) = v28;
          v7 = v130;
          *(_DWORD *)(v130 + 64) = ((__int64)((unsigned __int64)a1[18] << 16) / (int)(v27 / 0xAC44 * v28)) >> 8;
        }
        else if ( v19 != 0 )
        {
          *(_DWORD *)(v7 + 72) = v19;
        }
        goto LABEL_74;
      default:
        goto LABEL_74;
    }
  }
}

