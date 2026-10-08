/* ea:   0x000690F0 */
/* name: sub_690F0 */
/* size: 1560 */
int __fastcall sub_690F0(
        int *a1,
        unsigned int a2,
        unsigned int a3,
        int a4,
        int a5,
        _DWORD *a6,
        unsigned int *a7,
        int a8,
        unsigned int *a9,
        unsigned int *a10)
{
  int *v10; // r6
  int v12; // r9
  int *v13; // r3
  unsigned int v14; // r4
  int v15; // t1
  _DWORD *v17; // r1
  unsigned int v18; // r4
  unsigned int v19; // r8
  int *v20; // r1
  unsigned int v21; // r0
  int v22; // r3
  _DWORD *v23; // r2
  int v24; // r1
  int v25; // r0
  int *v26; // r1
  int v27; // r2
  int v28; // r3
  unsigned int v29; // r4
  int v30; // r2
  unsigned int *v31; // r3
  unsigned int v32; // r2
  int v33; // r0
  int v34; // t1
  int v35; // r0
  int *v36; // r1
  int v37; // r2
  int v38; // lr
  int v39; // r11
  _DWORD *v40; // r1
  int v41; // r8
  unsigned int v42; // r0
  signed int v43; // r7
  char v44; // r6
  unsigned int v45; // r5
  unsigned int v46; // r3
  int v47; // r1
  _DWORD *v48; // r2
  unsigned int v49; // r0
  unsigned int v50; // t1
  unsigned int v51; // r1
  int v52; // r0
  unsigned int v53; // r2
  int v54; // r1
  char v55; // r1
  int v56; // r7
  unsigned int v57; // r3
  int v58; // r3
  int v59; // r3
  unsigned int v60; // r4
  _DWORD *v61; // r1
  unsigned int v62; // r4
  unsigned int i; // r0
  int v64; // r4
  _DWORD *v65; // r3
  int v66; // t1
  bool v67; // zf
  unsigned int v68; // [sp+0h] [bp-130h]
  signed int v69; // [sp+4h] [bp-12Ch]
  unsigned int *v70; // [sp+8h] [bp-128h]
  unsigned int v71; // [sp+Ch] [bp-124h]
  int v72; // [sp+10h] [bp-120h]
  char v73; // [sp+14h] [bp-11Ch]
  int v74; // [sp+18h] [bp-118h]
  int v75; // [sp+1Ch] [bp-114h]
  _DWORD *v76; // [sp+20h] [bp-110h]
  int v77; // [sp+28h] [bp-108h]
  int v78; // [sp+2Ch] [bp-104h]
  unsigned int v79; // [sp+30h] [bp-100h]
  int v80; // [sp+38h] [bp-F8h]
  int v81; // [sp+3Ch] [bp-F4h]
  unsigned int v84; // [sp+48h] [bp-E8h]
  int v85; // [sp+50h] [bp-E0h] BYREF
  _DWORD v86[14]; // [sp+54h] [bp-DCh] BYREF
  int v87; // [sp+8Ch] [bp-A4h] BYREF
  _DWORD v88[16]; // [sp+90h] [bp-A0h] BYREF
  _DWORD v89[24]; // [sp+D0h] [bp-60h] BYREF

  v10 = a1;
  v12 = 0;
  v13 = a1;
  v85 = 0;
  v14 = a2;
  memset(v86, 0, sizeof(v86));
  v87 = 0;
  do
  {
    v15 = *v13++;
    --v14;
    ++v86[v15 - 1];
  }
  while ( v14 != 0 );
  if ( v85 == a2 )
  {
    *a6 = 0;
    *a7 = 0;
    return 0;
  }
  v17 = v86;
  v68 = *a7;
  v18 = 1;
  while ( *v17 == 0 )
  {
    if ( v17[1] != 0 )
    {
      ++v18;
      break;
    }
    if ( v17[2] != 0 )
    {
      v18 += 2;
      break;
    }
    if ( v17[3] != 0 )
    {
      v18 += 3;
      break;
    }
    if ( v17[4] != 0 )
    {
      v18 += 4;
      break;
    }
    v18 += 5;
    v17 += 5;
    if ( v18 > 0xF )
      break;
  }
  v19 = 15;
  v69 = v18;
  v75 = 15;
  if ( v68 < v18 )
    v68 = v18;
  v20 = &v87;
  do
  {
    if ( *v20 != 0 )
      break;
    v75 = --v19;
    --v20;
  }
  while ( v19 != 0 );
  if ( v68 <= v19 )
  {
    v21 = v68;
  }
  else
  {
    v21 = v19;
    v68 = v19;
  }
  *a7 = v21;
  v22 = 1 << v18;
  if ( v18 < v19 )
  {
    v23 = &v86[v18 - 1];
    do
    {
      v24 = v22 - *v23;
      if ( v24 < 0 )
        return -3;
      ++v18;
      ++v23;
      v22 = 2 * v24;
    }
    while ( v18 < v19 );
  }
  v26 = &v86[v19 - 1];
  v25 = *v26;
  v27 = v22 - *v26;
  v80 = v27;
  if ( v27 < 0 )
    return -3;
  v88[1] = 0;
  *v26 = v25 + v27;
  v28 = 0;
  v29 = v19 - 1;
  if ( v19 != 1 )
  {
    v30 = 0;
    do
    {
      v28 += v86[v30];
      --v29;
      v88[v30 + 2] = v28;
      ++v30;
    }
    while ( v29 != 0 );
  }
  v31 = a10;
  v32 = 0;
  do
  {
    v34 = *v10++;
    v33 = v34;
    if ( v34 != 0 )
    {
      v36 = &v88[v33];
      v35 = *v36;
      a10[v35] = v32;
      *v36 = v35 + 1;
    }
    ++v32;
  }
  while ( v32 < a2 );
  v37 = v69;
  v71 = 0;
  v81 = v88[v19];
  v70 = a10;
  v38 = -1;
  v88[0] = 0;
  v39 = -v68;
  v89[0] = 0;
  v79 = 0;
  v74 = 0;
  if ( v69 <= (int)v19 )
  {
    v73 = v69 - 1;
    v40 = &v86[v69 - 1];
    v76 = v40;
    v72 = v78;
    while ( 1 )
    {
      v41 = *v40;
      if ( *v40 != 0 )
        break;
LABEL_77:
      ++v37;
      v19 = v75;
      ++v40;
      v69 = v37;
      v76 = v40;
      ++v73;
      if ( v37 > v75 )
      {
        v12 = 0;
        goto LABEL_79;
      }
      v31 = a10;
    }
    while ( 1 )
    {
      v42 = v68;
      --v41;
      v43 = v39 + v68;
      if ( v37 > (int)(v39 + v68) )
        break;
      v55 = v69;
LABEL_59:
      BYTE1(v77) = v55 - v39;
      if ( v70 < &v31[v81] )
      {
        v57 = *v70;
        v72 = *v70;
        if ( *v70 >= a3 )
        {
          v58 = v57 - a3;
          LOBYTE(v77) = *(_BYTE *)(a5 + 4 * v58) + 80;
          v56 = *(_DWORD *)(a4 + 4 * v58);
          v72 = v56;
        }
        else
        {
          v56 = *v70;
          if ( v57 >= 0x100 )
            LOBYTE(v77) = 96;
          else
            LOBYTE(v77) = 0;
        }
        ++v70;
      }
      else
      {
        v56 = v72;
        LOBYTE(v77) = -64;
      }
      v59 = 1 << (v69 - v39);
      v60 = v71 >> v39;
      if ( v71 >> v39 < v79 )
      {
        v61 = (_DWORD *)(v74 + 8 * v60);
        do
        {
          v60 += v59;
          *v61 = v77;
          v61[1] = v56;
          v61 += 2 * v59;
        }
        while ( v60 < v79 );
      }
      v62 = v71;
      for ( i = 1 << v73; (i & v62) != 0; i >>= 1 )
        v62 ^= i;
      v64 = i ^ v62;
      v71 = v64;
      v65 = &v88[v38];
      if ( (((1 << v39) - 1) & v64) != *v65 )
      {
        do
        {
          --v38;
          v39 -= v68;
          v66 = *--v65;
        }
        while ( (((1 << v39) - 1) & v64) != v66 );
      }
      if ( v41 == 0 )
      {
        v40 = v76;
        v37 = v69;
        goto LABEL_77;
      }
      v31 = a10;
      v37 = v69;
    }
    v44 = v39 - v68;
    while ( 1 )
    {
      v39 += v42;
      v45 = v37 - v39;
      v46 = v75 - v39;
      if ( v75 - v39 > v42 )
        v46 = v42;
      v44 += v42;
      v43 += v42;
      ++v38;
      if ( 1 << v45 > (unsigned int)(v41 + 1) )
      {
        v47 = (1 << v45) - v41 - 1;
        v48 = v76;
        if ( v45 < v46 && ++v45 < v46 )
        {
          do
          {
            v50 = v48[1];
            ++v48;
            v49 = v50;
            v51 = 2 * v47;
            if ( v51 <= v50 )
              break;
            ++v45;
            v47 = v51 - v49;
          }
          while ( v45 < v46 );
        }
      }
      v79 = 1 << v45;
      v84 = *a9 + (1 << v45);
      if ( v84 > 0x5A0 )
        return -3;
      v52 = a8 + 8 * *a9;
      v74 = v52;
      v89[v38] = v52;
      *a9 = v84;
      if ( v38 != 0 )
      {
        v88[v38] = v71;
        LOWORD(v77) = (unsigned __int8)v45 | ((unsigned __int8)v68 << 8);
        v53 = v71 >> v44;
        v54 = v89[v38 - 1];
        v72 = ((v52 - v54) >> 3) - (v71 >> v44);
        *(_DWORD *)(v54 + 8 * v53) = v77;
        *(_DWORD *)(v54 + 8 * v53 + 4) = v72;
      }
      else
      {
        *a6 = v52;
      }
      v55 = v69;
      if ( v69 <= v43 )
      {
        v31 = a10;
        goto LABEL_59;
      }
      v37 = v69;
      v42 = v68;
    }
  }
LABEL_79:
  v67 = v80 == 0;
  if ( v80 != 0 )
    v67 = v19 == 1;
  if ( !v67 )
    return -5;
  return v12;
}

