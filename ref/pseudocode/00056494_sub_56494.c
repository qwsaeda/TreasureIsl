/* ea:   0x00056494 */
/* name: sub_56494 */
/* size: 1160 */
BOOL __fastcall sub_56494(int a1, int a2)
{
  char v5; // r0
  unsigned __int8 v6; // r11
  __int16 v7; // r9
  __int16 v8; // r10
  __int16 v9; // r7
  __int16 v10; // r8
  int v11; // r5
  unsigned int v12; // r6
  int v13; // r4
  _DWORD *v14; // r4
  int v15; // t1
  int v16; // r1
  unsigned __int16 v17; // r4
  unsigned int v18; // r5
  int v19; // r7
  int v20; // r8
  unsigned __int16 v21; // r9
  unsigned __int16 v22; // r1
  int v23; // r2
  unsigned __int16 v24; // r3
  unsigned __int16 v25; // r2
  __int64 v26; // r2
  unsigned __int16 v27; // r1
  _WORD *v28; // r5
  unsigned int v29; // r2
  __int16 v30; // r0
  unsigned __int16 v31; // r0
  unsigned __int16 v32; // r0
  __int16 v33; // r4
  unsigned int v34; // r2
  unsigned int v35; // r1
  __int16 v36; // r0
  unsigned __int16 v37; // [sp+4h] [bp-48h]
  unsigned __int16 v38; // [sp+8h] [bp-44h]
  unsigned __int16 v39; // [sp+Ch] [bp-40h]
  int v41; // [sp+14h] [bp-38h]
  unsigned __int16 *v42; // [sp+18h] [bp-34h]
  int v44; // [sp+24h] [bp-28h]

  if ( a2 == 0 )
    return false;
  v42 = (unsigned __int16 *)(a1 + 136);
  v39 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  v41 = 10 * v39;
  v44 = 10 * *(unsigned __int16 *)(a2 + 22);
  v37 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
  v38 = *(_WORD *)(a2 + 24);
  v5 = 2;
  if ( *(_DWORD *)(a1 + 176) != 8 )
    v5 = 1;
  v6 = v5;
  v7 = 0;
  v8 = 0;
  if ( (unsigned int)*(unsigned __int8 *)(a2 + 49) - 1 <= 3 )
  {
    v7 = 350;
    v8 = 60;
  }
  v9 = 0;
  v10 = 0;
  v11 = 0;
  v12 = 0;
  while ( v11 < 2 )
  {
    v13 = *(_DWORD *)(a1 + 20);
    v15 = *(_DWORD *)(v13 + 28);
    v14 = (_DWORD *)(v13 + 28);
    if ( v12 >= (v14[1] - v15) >> 2 )
      TerminateProcess((HANDLE)0x42, 0);
    v16 = *(_DWORD *)(*v14 + 4 * v12);
    if ( v16 == 4 )
    {
      v10 = 1;
      goto LABEL_15;
    }
    if ( v16 == 6 )
    {
      v9 = 1;
LABEL_15:
      ++v11;
    }
    if ( (int)++v12 >= 5 )
      break;
  }
  v17 = 50 * v9 + v7;
  v18 = v37;
  v19 = 100 - (unsigned __int16)(50 * v10 + v8);
  v20 = v44 - 2 * (v37 + v39) * v6 * v19 / 100;
  if ( v20 <= 0 )
  {
LABEL_30:
    *(_WORD *)(a2 + 24) = 0;
    sub_31200(a2, *(_WORD *)(a2 + 22));
    v27 = sub_16C9C(v42, 1) - v37;
    v28 = (_WORD *)(a1 + 140);
    v29 = *(unsigned __int16 *)(a1 + 140);
    if ( v27 < v29 )
      *v28 = v29 - v27;
    else
      *v28 = 0;
    v30 = sub_16C9C(v42, 0);
    sub_16D78(v42, v30 - v39);
  }
  else
  {
    while ( 1 )
    {
      v21 = v20 / 10 + 1;
      if ( v21 > 0x3E7u )
        v21 = 999;
      if ( v21 > (unsigned int)v38 )
      {
        v22 = v38;
      }
      else
      {
        v22 = v21;
        v38 = v21;
      }
      v23 = v41 - 2 * (v21 + v22) * v17 / 100;
      v41 = v23;
      if ( v23 <= 0 )
        break;
      v24 = v23 / 10 + 1;
      v39 = v24;
      if ( v24 > 0x3E7u )
      {
        v24 = 999;
        v39 = 999;
      }
      if ( v18 < v24 )
      {
        v25 = v37;
      }
      else
      {
        v25 = v24;
        v37 = v24;
      }
      v18 = v25;
      v26 = 2748779070LL * (v25 + v24) * v6 * v19;
      v20 -= (SHIDWORD(v26) >> 5) + (HIDWORD(v26) >> 31);
      if ( v20 <= 0 )
        goto LABEL_30;
    }
    v31 = sub_16C9C(v42, 1);
    sub_357C0((_WORD *)(a1 + 140), v31);
    v32 = sub_16C9C(v42, 0);
    sub_16D78(v42, v32);
    v33 = *(_WORD *)(a2 + 22);
    v34 = *(unsigned __int16 *)(a2 + 24);
    v35 = (unsigned __int16)(v33 - v38);
    v36 = 0;
    if ( v35 <= v34 )
      v36 = v34 - v35;
    *(_WORD *)(a2 + 24) = v36;
    sub_31200(a2, v33 - v21);
  }
  return v41 > v20;
}

