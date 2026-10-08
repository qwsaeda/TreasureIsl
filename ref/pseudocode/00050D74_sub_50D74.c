/* ea:   0x00050D74 */
/* name: sub_50D74 */
/* size: 956 */
int __fastcall sub_50D74(_DWORD *a1, int *a2, int *a3, int a4)
{
  _DWORD *v4; // r7
  int v5; // r0
  _DWORD *v6; // r11
  int v7; // r6
  int v8; // r10
  int v9; // r7
  int v10; // r6
  int v11; // r8
  int v12; // t1
  int v13; // r4
  int v14; // r3
  _DWORD *v15; // r1
  char v16; // r0
  int v18; // r5
  int v19; // r0
  unsigned __int16 *v20; // r5
  unsigned __int16 v21; // t1
  int v22; // r4
  int v23; // r0
  int v24; // r4
  int v25; // r0
  int v26; // r4
  int v27; // r0
  int v28; // r4
  int v29; // r0
  int v30; // r3
  int v31; // r1
  int v32; // r0
  int v33; // r1
  int v34; // r0
  int v35; // r1
  int v36; // r0
  int *v37; // r2
  int v38; // r1
  int *v39; // r4
  int v40; // r1
  int v41; // r0

  v4 = a1;
  v5 = a1[30];
  if ( v5 != 0 )
  {
    if ( v5 == 8 )
    {
      v30 = v4[11];
      if ( v4[10] == v30 )
      {
        if ( (*(_BYTE *)(v4[3] + 23) & 0x10) != 0 )
          goto LABEL_37;
        v31 = v4[26];
        v4[10] = v4[12];
        v32 = v4[22];
        v4[24] = v31;
        v4[18] = v32;
        v33 = v4[23];
        v4[27] = 0;
        v34 = v4[9];
        v4[19] = v33;
      }
      else
      {
        if ( (*(_BYTE *)(v4[3] + 23) & 0x10) != 0x10 )
          goto LABEL_37;
        v35 = v4[20];
        v4[24] = v4[25];
        v36 = v4[21];
        v4[18] = v35;
        v4[19] = v36;
        v34 = v4[8];
        v4[10] = v30;
        v4[27] = 0;
      }
      v4[7] = v34;
    }
  }
  else
  {
    v6 = v4 + 46;
    if ( *(_DWORD *)v4[46] == v4[46] )
      return 0;
    v7 = v4[6];
    v8 = v4[46];
    v9 = 0;
    v12 = *(_DWORD *)(v7 + 52);
    v10 = v7 + 52;
    v11 = v12;
    if ( v12 <= 0 )
    {
LABEL_10:
      v16 = 0;
    }
    else
    {
      v13 = *(_DWORD *)(v10 + 4);
      while ( 1 )
      {
        v14 = 0;
        if ( v13 > 0 )
          break;
LABEL_9:
        if ( ++v9 >= v11 )
          goto LABEL_10;
      }
      v15 = *(_DWORD **)(*(_DWORD *)(v10 + 8) + 4 * v9);
      while ( *(_DWORD *)(*(_DWORD *)v8 + 8) != *v15 )
      {
        ++v14;
        ++v15;
        if ( v14 >= v13 )
          goto LABEL_9;
      }
      v16 = 1;
    }
    if ( v16 == 0 )
    {
      sub_52CE8(v6);
      return 0;
    }
    if ( (*(_BYTE *)(*(_DWORD *)(*(_DWORD *)v8 + 8) + 22) & 3) != 1 )
    {
      sub_52CE8(v6);
      return 0;
    }
    v4 = a1;
    a1[3] = 0;
    a1[3] = *(_DWORD *)(*(_DWORD *)v8 + 8);
    sub_52958(v6);
    v18 = a1[3];
    v19 = *(_DWORD *)(v18 + 36);
    a1[30] = 16;
    a1[5] = v19;
    v21 = *(_WORD *)(v18 + 20);
    v20 = (unsigned __int16 *)(v18 + 20);
    v22 = a1[17] - 1;
    if ( v22 == v21 / a4 )
    {
      v23 = a1[31];
      a1[17] = v22;
      if ( v23 != 2 )
      {
        a1[27] = 0;
        a1[31] = 2;
      }
    }
    v24 = a1[16] + 1;
    if ( v24 == *v20 % a4 )
    {
      v25 = a1[31];
      a1[16] = v24;
      if ( v25 != 3 )
      {
        a1[27] = 0;
        a1[31] = 3;
      }
    }
    v26 = a1[17] + 1;
    if ( v26 == *v20 / a4 )
    {
      v27 = a1[31];
      a1[17] = v26;
      if ( v27 != 1 )
      {
        a1[27] = 0;
        a1[31] = 1;
      }
    }
    v28 = a1[16] - 1;
    if ( v28 == *v20 % a4 )
    {
      v29 = a1[31];
      a1[16] = v28;
      if ( v29 != 0 )
      {
        a1[31] = 0;
        a1[27] = 0;
      }
    }
  }
LABEL_37:
  switch ( v4[31] )
  {
    case 0:
      v39 = a2;
      v40 = *a2 + 2;
      goto LABEL_42;
    case 1:
      v39 = a2;
      v40 = *a2 - 2;
LABEL_42:
      v37 = a3;
      *v39 = v40;
      v38 = *a3 - 1;
      goto LABEL_43;
    case 2:
      *a2 += 2;
      v37 = a3;
      v38 = *a3 + 1;
      goto LABEL_43;
    case 3:
      v37 = a3;
      *a2 -= 2;
      v38 = *a3 + 1;
LABEL_43:
      *v37 = v38;
      break;
    default:
      break;
  }
  v41 = v4[6];
  --v4[30];
  sub_3958C((_DWORD **)(v41 + 64));
  return 1;
}

