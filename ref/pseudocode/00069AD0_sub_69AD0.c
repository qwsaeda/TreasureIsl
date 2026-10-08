/* ea:   0x00069AD0 */
/* name: sub_69AD0 */
/* size: 1240 */
int __fastcall sub_69AD0(int a1, int a2, unsigned int a3, int a4, _DWORD *a5, unsigned __int8 **a6)
{
  unsigned int v7; // r7
  unsigned int v9; // lr
  unsigned int v10; // r0
  unsigned int v11; // r3
  bool v12; // cf
  unsigned int v13; // r6
  int v14; // r9
  int v15; // r5
  unsigned int v16; // r1
  unsigned __int8 *v17; // r4
  int v18; // t1
  unsigned __int8 *v19; // r4
  int v20; // r8
  int v21; // r0
  unsigned int v22; // lr
  unsigned int v23; // r6
  int v24; // r0
  int v25; // r1
  unsigned int v26; // r4
  unsigned int v27; // r3
  unsigned int v28; // r8
  unsigned __int8 *v29; // r1
  int v30; // t1
  int v31; // r2
  unsigned __int8 *v32; // r6
  int v33; // r1
  unsigned int v34; // lr
  unsigned int v35; // r4
  int v36; // r1
  unsigned int v37; // r3
  unsigned __int8 *v38; // r1
  int v39; // t1
  int v40; // r1
  int v41; // r0
  unsigned int v42; // r1
  char *v43; // r3
  unsigned int v44; // r1
  int v45; // r2
  int v46; // r1
  int v47; // r2
  char v48; // t1
  char *v49; // r1
  char v50; // t1
  char *v51; // r3
  char v52; // t1
  unsigned int v53; // r2
  char *v54; // r3
  char v55; // t1
  char v56; // t1
  char *v57; // r3
  char v58; // t1
  unsigned int v59; // r2
  char *v60; // r3
  char v61; // t1
  char v62; // t1
  unsigned int v63; // r1
  unsigned int v64; // r2
  unsigned __int8 *v65; // r0
  unsigned __int8 *v66; // r1
  unsigned __int8 **v68; // r8
  unsigned __int8 *v69; // r1
  unsigned int v70; // r3
  unsigned int v71; // r2
  _DWORD *v72; // r5
  unsigned __int8 *v73; // r4
  unsigned int v74; // r0
  bool v75; // zf
  unsigned int v76; // r2
  unsigned __int8 *v77; // r0
  unsigned __int8 *v78; // r1
  unsigned __int8 *v79; // r1
  unsigned __int8 *v80; // r0
  unsigned __int8 *v81; // r1
  _BYTE *v82; // [sp+0h] [bp-38h]
  unsigned __int8 *v83; // [sp+4h] [bp-34h]
  unsigned int v84; // [sp+8h] [bp-30h]
  unsigned int v85; // [sp+Ch] [bp-2Ch]
  unsigned int v86; // [sp+10h] [bp-28h]
  unsigned int v87; // [sp+10h] [bp-28h]

  v7 = a3;
  v9 = a5[13];
  v83 = *a6;
  v86 = a5[8];
  v10 = a5[12];
  v11 = (unsigned int)a6[1];
  v12 = v9 >= v10;
  v13 = a5[7];
  if ( v9 < v10 )
    v10 -= v9;
  v14 = dword_80FE8[a2];
  if ( v12 )
    v10 = a5[11];
  else
    a3 = v10 - 1;
  v15 = dword_80FE8[a1];
  if ( v12 )
    a3 = v10 - v9;
  v85 = (unsigned int)a6[1];
  v82 = (_BYTE *)a5[13];
  v84 = a3;
  while ( 1 )
  {
    v16 = v86;
    if ( v13 < 0x14 )
    {
      v17 = v83;
      do
      {
        v18 = *v17++;
        v85 = --v11;
        v16 |= v18 << v13;
        v83 = v17;
        v13 += 8;
      }
      while ( v13 < 0x14 );
    }
    v19 = (unsigned __int8 *)(v7 + 8 * (v15 & v16));
    v20 = *v19;
    v21 = v19[1];
    if ( *v19 != 0 )
      break;
    v13 -= v21;
    --a3;
    v86 = v16 >> v21;
    *(_BYTE *)v9++ = *((_DWORD *)v19 + 1);
    v84 = a3;
    v82 = (_BYTE *)v9;
LABEL_44:
    if ( a3 < 0x102 || v11 < 0xA )
    {
      v64 = (unsigned int)&a6[1][-v11];
      if ( v13 >> 3 < v64 )
        v64 = v13 >> 3;
      a5[7] = v13 - 8 * v64;
      a5[8] = v86;
      v65 = *a6;
      v66 = a6[2];
      *a6 = &v83[-v64];
      a6[1] = (unsigned __int8 *)(v64 + v11);
      a6[2] = &v83[v66 - v65 - v64];
      a5[13] = v9;
      return 0;
    }
  }
  v22 = v13 - v21;
  v23 = v16 >> v21;
  if ( (v20 & 0x10) != 0 )
  {
LABEL_18:
    v25 = *((_DWORD *)v19 + 1);
    v26 = v23 >> (v20 & 0xF);
    v27 = v22 - (v20 & 0xF);
    v28 = (dword_80FE8[v20 & 0xF] & v23) + v25;
    if ( v27 < 0xF )
    {
      v29 = v83;
      do
      {
        --v85;
        v30 = *v29++;
        v26 |= v30 << v27;
        v83 = v29;
        v27 += 8;
      }
      while ( v27 < 0xF );
    }
    v32 = (unsigned __int8 *)(a4 + 8 * (v14 & v26));
    v31 = *v32;
    v33 = v32[1];
    v34 = v26 >> v33;
    v35 = v27 - v33;
    v87 = v27 - v33;
    if ( (v31 & 0x10) == 0 )
    {
      while ( (v31 & 0x40) == 0 )
      {
        v32 += 8 * (dword_80FE8[v31] & v34) + 8 * *((_DWORD *)v32 + 1);
        v31 = *v32;
        v36 = v32[1];
        v35 -= v36;
        v34 >>= v36;
        v87 = v35;
        if ( (v31 & 0x10) != 0 )
          goto LABEL_24;
      }
      v68 = a6;
      v69 = a6[1];
      v70 = v85;
      a6[6] = (unsigned __int8 *)aInvalidDistanc;
      v71 = (unsigned int)&v69[-v85];
      v72 = a5;
      if ( v35 >> 3 < (unsigned int)&v69[-v85] )
        v71 = v35 >> 3;
      a5[8] = v34;
      v73 = &v83[-v71];
      v74 = v87 - 8 * v71;
      goto LABEL_59;
    }
LABEL_24:
    v37 = v31 & 0xF;
    if ( v35 < v37 )
    {
      v38 = v83;
      do
      {
        --v85;
        v39 = *v38++;
        v34 |= v39 << v35;
        v83 = v38;
        v35 += 8;
      }
      while ( v35 < v37 );
    }
    v40 = *((_DWORD *)v32 + 1);
    v13 = v35 - v37;
    v41 = (dword_80FE8[v37] & v34) + v40;
    v42 = v34 >> v37;
    v9 = (unsigned int)v82;
    v86 = v42;
    v43 = &v82[-v41];
    v84 -= v28;
    v44 = a5[10];
    if ( (unsigned int)&v82[-v41] >= v44 )
    {
      v58 = *v43;
      v57 = v43 + 1;
      v59 = v28 - 2;
      *v82 = v58;
      v61 = *v57;
      v60 = v57 + 1;
      v82[1] = v61;
      v9 = (unsigned int)(v82 + 2);
      do
      {
        v62 = *v60++;
        --v59;
        *(_BYTE *)v9++ = v62;
        v82 = (_BYTE *)v9;
      }
      while ( v59 != 0 );
      a3 = v84;
    }
    else
    {
      v45 = a5[11];
      do
        v43 += v45 - v44;
      while ( (unsigned int)v43 < v44 );
      v46 = v45 - (_DWORD)v43;
      if ( v28 <= v45 - (int)v43 )
      {
        v52 = *v43;
        v51 = v43 + 1;
        v53 = v28 - 2;
        *v82 = v52;
        v55 = *v51;
        v54 = v51 + 1;
        v82[1] = v55;
        v9 = (unsigned int)(v82 + 2);
        do
        {
          v56 = *v54++;
          --v53;
          *(_BYTE *)v9++ = v56;
          v82 = (_BYTE *)v9;
        }
        while ( v53 != 0 );
        a3 = v84;
      }
      else
      {
        v47 = v28 - v46;
        do
        {
          v48 = *v43++;
          --v46;
          *(_BYTE *)v9++ = v48;
        }
        while ( v46 != 0 );
        v49 = (char *)a5[10];
        do
        {
          v50 = *v49++;
          --v47;
          *(_BYTE *)v9++ = v50;
          v82 = (_BYTE *)v9;
        }
        while ( v47 != 0 );
        a3 = v84;
      }
    }
LABEL_43:
    v11 = v85;
    goto LABEL_44;
  }
  while ( (v20 & 0x40) == 0 )
  {
    v19 += 8 * (dword_80FE8[v20] & v23) + 8 * *((_DWORD *)v19 + 1);
    v24 = v19[1];
    v20 = *v19;
    if ( *v19 == 0 )
    {
      v63 = v23 >> v24;
      v13 = v22 - v24;
      a3 = v84 - 1;
      v86 = v63;
      *v82 = *((_DWORD *)v19 + 1);
      v9 = (unsigned int)(v82 + 1);
      --v84;
      ++v82;
      goto LABEL_43;
    }
    v23 >>= v24;
    v22 -= v24;
    if ( (v20 & 0x10) != 0 )
      goto LABEL_18;
  }
  v75 = (v20 & 0x20) == 0;
  v70 = v85;
  v68 = a6;
  if ( v75 )
  {
    v79 = a6[1];
    a6[6] = (unsigned __int8 *)aInvalidLiteral;
    v71 = (unsigned int)&v79[-v85];
    v72 = a5;
    if ( v22 >> 3 < (unsigned int)&v79[-v85] )
      v71 = v22 >> 3;
    a5[8] = v23;
    v73 = &v83[-v71];
    v74 = v22 - 8 * v71;
LABEL_59:
    v72[7] = v74;
    v80 = *v68;
    v81 = v68[2];
    v68[1] = (unsigned __int8 *)(v71 + v70);
    *v68 = v73;
    v68[2] = &v73[v81 - v80];
    v72[13] = v82;
    return -3;
  }
  v76 = (unsigned int)&a6[1][-v85];
  a5[8] = v23;
  if ( v22 >> 3 < v76 )
    v76 = v22 >> 3;
  a5[7] = v22 - 8 * v76;
  v77 = *a6;
  v78 = a6[2];
  *a6 = &v83[-v76];
  a6[1] = (unsigned __int8 *)(v76 + v85);
  a6[2] = &v83[v78 - v77 - v76];
  a5[13] = v82;
  return 1;
}

