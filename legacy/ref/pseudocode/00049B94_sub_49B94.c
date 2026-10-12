/* ea:   0x00049B94 */
/* name: sub_49B94 */
/* size: 1820 */
int __fastcall sub_49B94(_DWORD *a1, int *a2, _WORD *a3)
{
  _DWORD *v3; // r8
  int *v4; // r9
  unsigned __int8 **v5; // r7
  int result; // r0
  int v7; // r0
  char v8; // r6
  int v9; // r5
  int v10; // r4
  int v11; // r1
  char v12; // r0
  unsigned int v13; // r1
  int v14; // r0
  unsigned __int8 *v15; // r4
  _DWORD *v16; // r3
  _DWORD *v17; // r4
  int v18; // r0
  int v19; // r10
  int v20; // r11
  int v21; // r9
  int v22; // r0
  int v23; // r5
  _DWORD *v24; // r3
  int v25; // r2
  int v26; // r0
  _DWORD *v27; // r8
  int v28; // r4
  char v29; // r0
  int v30; // r0
  int v31; // r0
  int v32; // r4
  int v33; // r0
  int v34; // r5
  int v35; // r3
  void **v36; // r4
  int v37; // r0
  int v38; // r6
  _DWORD *v39; // r7
  unsigned __int8 *i; // r4
  int v41; // r3
  char v42; // r6
  char v43; // r0
  char v44; // r0
  unsigned int v45; // r1
  char v46; // r6
  char v47; // r0
  char v48; // r0
  unsigned int v49; // r1
  char v50; // r6
  _DWORD *v51; // r1
  char v52; // r0
  char v53; // r0
  unsigned int v54; // r1
  char v55; // [sp+0h] [bp-6Ch] BYREF
  char v56[3]; // [sp+1h] [bp-6Bh] BYREF
  int v57; // [sp+4h] [bp-68h] BYREF
  int v58; // [sp+8h] [bp-64h]
  int v59; // [sp+Ch] [bp-60h] BYREF
  _WORD *v60; // [sp+10h] [bp-5Ch]
  int *v61; // [sp+14h] [bp-58h]
  int v62; // [sp+18h] [bp-54h]
  _DWORD *v63; // [sp+1Ch] [bp-50h] BYREF
  int v64; // [sp+20h] [bp-4Ch] BYREF
  int v65; // [sp+24h] [bp-48h] BYREF
  int v66; // [sp+28h] [bp-44h] BYREF
  int v67; // [sp+2Ch] [bp-40h] BYREF
  _DWORD v68[2]; // [sp+30h] [bp-3Ch] BYREF
  _DWORD *v69; // [sp+38h] [bp-34h] BYREF
  _BYTE *v70; // [sp+3Ch] [bp-30h]

  v3 = a1;
  v60 = a3;
  v4 = a2;
  v63 = a1;
  v61 = a2;
  *a2 = 0;
  v5 = (unsigned __int8 **)(a1 + 41);
  *a3 = 0;
  if ( a1[44] != 2 )
  {
    result = rand() % 10;
    if ( result + 1 > v5[1] - *v5 )
      return result;
  }
  v7 = v3[44];
  v8 = 1;
  v57 = 0;
  v9 = 1;
  v58 = 1;
  if ( v7 == 2 )
  {
    sub_4A34C(&v69, &v55);
    v64 = 1;
    sub_406DC((int *)&v69, &v64);
    v62 = 2;
    v57 = 2;
    while ( 1 )
    {
      v10 = *sub_1DE18((_DWORD *)(v3[6] + 496), &v63);
      v11 = *sub_1E48C(v3[6] + 496, &v67, (int)&v57);
      v12 = v8;
      if ( v11 == v10 )
        v12 = 0;
      if ( v12 != 0 )
        sub_406DC((int *)&v69, &v57);
      if ( ++v57 > 4 )
        break;
      v8 = v58;
    }
    v13 = rand() % (unsigned int)((v70 - (_BYTE *)v69) >> 2);
    v14 = v3[2];
    v15 = *v5;
    v57 = v69[v13];
    if ( v57 == v14 )
      v9 = v62;
    while ( v15 != v5[1] )
    {
      v16 = (_DWORD *)sub_173C4(v3[6] + 652, *v15);
      *v4 = (int)v16;
      if ( v16[13] != 7 && v16[8] != 4 && v16[14] == v9 )
        break;
      *v4 = 0;
      ++v15;
    }
    sub_30BC4((void **)&v69);
    goto LABEL_19;
  }
  v19 = 0;
  v20 = 0;
  v21 = 4;
  v22 = sub_16C9C((unsigned __int16 *)(v3[6] + 424), 2);
  v65 = 1;
  v23 = v58;
  v24 = (_DWORD *)(*(_DWORD *)(*(_DWORD *)sub_2F944((_DWORD *)(v3[6] + 520), &v65) + 8) + v22);
  v25 = (int)*(&off_6FE14 + dword_7FB6C);
  v69 = v24;
  v62 = 2;
  if ( (int)v24 >= v25 )
  {
    v3 = (_DWORD *)v58;
    v57 = v58;
  }
  v26 = 2;
  if ( (int)v24 < v25 )
  {
    v3 = (_DWORD *)v57;
    v20 = v58;
    v19 = (int)v24;
  }
  v59 = 2;
  if ( v3 == nullptr )
  {
    while ( v26 < 5 )
    {
      v27 = v63;
      v28 = *sub_1DE18((_DWORD *)(v63[6] + 496), &v67);
      if ( *sub_1E48C(v27[6] + 496, v68, (int)&v59) == v28 )
        v29 = 0;
      else
        v29 = v23;
      if ( v29 != 0 )
      {
        v30 = sub_1DD10((_DWORD *)(v27[6] + 496), &v59);
        v31 = sub_16C9C((unsigned __int16 *)(*(_DWORD *)v30 + 136), 2);
        v66 = v23;
        v32 = v31;
        v33 = sub_2F944((_DWORD *)(v27[6] + 520), &v66);
        v34 = v59;
        v35 = *(_DWORD *)(*(_DWORD *)v33 + 8) + v32;
        v36 = (void **)&(&v69)[v59];
        v37 = v27[2];
        *(v36 - 1) = (void *)v35;
        if ( v34 != v37 )
        {
          if ( v35 >= (int)*(&off_6FE14 + dword_7FB6C) )
          {
            v3 = (_DWORD *)v34;
            v57 = v34;
            v19 += v35;
            goto LABEL_43;
          }
          if ( v35 > (int)(&v69)[v20 - 1] )
            v20 = v34;
        }
        v3 = (_DWORD *)v57;
        v19 += (int)*(v36 - 1);
      }
      else
      {
        v34 = v59;
        v3 = (_DWORD *)v57;
        (&v69)[v59 - 1] = (_DWORD *)-1;
        --v21;
      }
LABEL_43:
      v26 = v34 + 1;
      v59 = v34 + 1;
      if ( v3 != nullptr )
        break;
      v23 = v58;
    }
  }
  if ( (int)(&v69)[v63[2] - 1] < v19 / v21 )
  {
    if ( v3 == nullptr )
      v57 = v20;
  }
  else if ( (int)v3 <= 0 )
  {
    v57 = v63[2];
    v38 = v58;
    goto LABEL_50;
  }
  v38 = v62;
LABEL_50:
  v3 = v63;
  v4 = v61;
  v39 = v63 + 41;
  for ( i = (unsigned __int8 *)v63[41]; i != (unsigned __int8 *)v39[1]; ++i )
  {
    v41 = sub_173C4(v3[6] + 652, *i);
    *v4 = v41;
    if ( *(_DWORD *)(v41 + 52) != 7 && *(_DWORD *)(v41 + 56) == v38 )
      break;
    *v4 = 0;
  }
LABEL_19:
  result = *v4;
  if ( *v4 == 0 )
    return result;
  sub_4A4E8(&v69, v56);
  v17 = (_DWORD *)*sub_1DDF0(v3[6] + 508, v68);
  v18 = *(_DWORD *)(*v4 + 32);
  switch ( v18 )
  {
    case 2:
      v50 = v58;
      while ( 1 )
      {
        v51 = (_DWORD *)*sub_1DE08((_DWORD *)(v3[6] + 508), v68);
        v52 = v50;
        if ( v17 == v51 )
          v52 = 0;
        if ( v52 == 0 )
          break;
        if ( *(unsigned __int8 *)(v17[5] + 50) == v57 )
          sub_4A2B0(&v69, v17 + 5);
        v17 = sub_16A30(v17);
      }
      v53 = v58;
      if ( v69 != (_DWORD *)v70 )
        v53 = 0;
      if ( v53 == 0 )
      {
        v54 = rand() % (unsigned int)((v70 - (_BYTE *)v69) >> 2);
        *v60 = *(_WORD *)(v69[v54] + 20);
        return sub_4A4FC(&v69);
      }
      break;
    case 3:
      v46 = v58;
      while ( 1 )
      {
        v47 = v46;
        if ( v17 == *(_DWORD **)(v3[6] + 508) )
          v47 = 0;
        if ( v47 == 0 )
          break;
        if ( *(unsigned __int8 *)(v17[5] + 50) == v57 )
          sub_4A2B0(&v69, v17 + 5);
        v17 = sub_16A30(v17);
      }
      v48 = v58;
      if ( v69 != (_DWORD *)v70 )
        v48 = 0;
      if ( v48 == 0 )
      {
        v49 = rand() % (unsigned int)((v70 - (_BYTE *)v69) >> 2);
        *v60 = *(unsigned __int8 *)(v69[v49] + 48);
        return sub_4A4FC(&v69);
      }
      break;
    case 4:
      v42 = v58;
      while ( 1 )
      {
        v43 = v42;
        if ( v17 == *(_DWORD **)(v3[6] + 508) )
          v43 = 0;
        if ( v43 == 0 )
          break;
        if ( *(_BYTE *)(v17[5] + 50) == 0 )
          sub_4A2B0(&v69, v17 + 5);
        v17 = sub_16A30(v17);
      }
      v44 = v58;
      if ( v69 != (_DWORD *)v70 )
        v44 = 0;
      if ( v44 == 0 )
      {
        v45 = rand() % (unsigned int)((v70 - (_BYTE *)v69) >> 2);
        *v60 = *(_WORD *)(*(_DWORD *)(v69[v45] + 4) + 20);
        return sub_4A4FC(&v69);
      }
      break;
    default:
      *v60 = v57;
      return sub_4A4FC(&v69);
  }
  *v61 = 0;
  return sub_4A4FC(&v69);
}

