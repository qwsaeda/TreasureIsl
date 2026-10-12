/* ea:   0x00048D00 */
/* name: sub_48D00 */
/* size: 1056 */
unsigned int __fastcall sub_48D00(int a1, _DWORD *a2, _DWORD *a3)
{
  int v5; // r5
  int v6; // r0
  const void *v7; // r1
  unsigned int result; // r0
  int v9; // r11
  _DWORD *i; // r4
  _DWORD *v11; // r9
  unsigned int *v12; // r7
  _DWORD *v13; // r6
  int v14; // r4
  int v15; // r0
  _DWORD *v16; // r5
  char *v17; // r7
  unsigned __int16 v18; // r6
  unsigned __int16 v19; // r8
  unsigned int v20; // r4
  int v21; // r2
  int v22; // r4
  unsigned int v23; // r5
  int v24; // r2
  int v25; // r0
  int v26; // r3
  unsigned int v27; // r0
  _DWORD *v28; // r1
  int v29; // r3
  unsigned int v30; // r0
  _DWORD *v31; // r1
  int v32; // r3
  unsigned int v33; // r0
  _DWORD *v34; // r1
  _BYTE v35[4]; // [sp+0h] [bp-4Ch] BYREF
  _DWORD *v36; // [sp+4h] [bp-48h] BYREF
  int v37; // [sp+8h] [bp-44h]
  int v38; // [sp+Ch] [bp-40h] BYREF
  unsigned int v39; // [sp+10h] [bp-3Ch]
  int v40; // [sp+14h] [bp-38h] BYREF
  _BYTE v41[8]; // [sp+18h] [bp-34h] BYREF
  unsigned int v42; // [sp+20h] [bp-2Ch] BYREF
  __int16 v43; // [sp+24h] [bp-28h]
  char v44; // [sp+26h] [bp-26h] BYREF

  v37 = a1;
  v5 = sub_16C9C((unsigned __int16 *)(a1 + 136), 2);
  v6 = a2[7];
  if ( v6 != 0 )
  {
    if ( v6 != 1 )
      goto LABEL_6;
    v7 = (char *)a2 + 22;
  }
  else
  {
    v7 = a2 + 4;
  }
  memcpy(v41, v7, 6u);
LABEL_6:
  result = (unsigned int)v41[0] * v5 / 0xA;
  v39 = result;
  if ( result >= 0x7D0 )
  {
    v43 = -1;
    v42 = -1;
    v9 = 3;
    sub_4A3A8(&v36, v35);
    v38 = 0;
    for ( i = (_DWORD *)*sub_1DDF0((int)a3, &v40); i != (_DWORD *)*sub_1DE08(a3, &v40); i = sub_16A30(i) )
    {
      v38 = i[5];
      if ( *(unsigned __int8 *)(v38 + 50) == *(_DWORD *)(v37 + 8) )
        sub_4A360(&v36, v41, v36, &v38);
    }
    sub_4ACB4(&v36, sub_4AC60);
    v11 = v36;
    v12 = &v42;
    v13 = (_DWORD *)*v36;
    do
    {
      if ( v13 == v11 )
        break;
      v14 = v13[2];
      if ( *(unsigned __int8 *)(v14 + 49) < 4u )
      {
        v15 = sub_17B98(v13[2]);
        v11 = v36;
        if ( *(unsigned __int8 *)(v14 + 49) + 1 <= v15 )
        {
          --v9;
          *(_WORD *)v12 = *(_WORD *)(v13[2] + 20);
          v12 = (unsigned int *)((char *)v12 + 2);
        }
      }
      v13 = (_DWORD *)*v13;
    }
    while ( v9 > 0 );
    v16 = (_DWORD *)*v11;
    if ( v9 <= 0 )
    {
      v19 = v42;
    }
    else
    {
      v17 = &v44 - 2 * v9;
      v18 = v43;
      v19 = v42;
      v20 = HIWORD(v42);
      do
      {
        if ( v16 == v11 )
          break;
        v21 = *(unsigned __int16 *)(v16[2] + 20);
        if ( v21 != v19 && v21 != (unsigned __int16)v20 && v21 != v18 )
        {
          *(_WORD *)v17 = *(_WORD *)(v16[2] + 20);
          v17 += 2;
          --v9;
          v18 = v43;
          v19 = v42;
          v20 = HIWORD(v42);
        }
        v16 = (_DWORD *)*v16;
      }
      while ( v9 > 0 );
    }
    v22 = 0;
    if ( v19 != 0xFFFF )
    {
      v23 = v39;
      do
      {
        if ( v22 >= 3 )
          break;
        v24 = *(_DWORD *)sub_1623C(a3, (unsigned __int16 *)&v42 + v22);
        v38 = v24;
        v25 = *(unsigned __int8 *)(v24 + 49);
        if ( v25 == 1 )
        {
          if ( v23 < 0x7D0 )
          {
            v22 = 3;
          }
          else
          {
            v32 = v37;
            v34 = (_DWORD *)(v37 + 144);
            v33 = *(_DWORD *)(v37 + 144);
            *v34 = v33 > 0x7D0 ? v33 - 2000 : 0;
            sub_17BC8(v24, (_DWORD *)(*(_DWORD *)(v32 + 24) + 32));
            v23 -= 2000;
          }
        }
        else if ( v25 == 2 )
        {
          if ( v23 >= 0xFA0 )
          {
            v29 = v37;
            v31 = (_DWORD *)(v37 + 144);
            v30 = *(_DWORD *)(v37 + 144);
            if ( v30 > 0xFA0 )
              *v31 = v30 - 4000;
            else
              *v31 = 0;
            sub_17BC8(v24, (_DWORD *)(*(_DWORD *)(v29 + 24) + 32));
            v23 -= 4000;
          }
        }
        else if ( v25 == 3 && v23 >= 0x2710 )
        {
          v26 = v37;
          v28 = (_DWORD *)(v37 + 144);
          v27 = *(_DWORD *)(v37 + 144);
          if ( v27 > 0x2710 )
            *v28 = v27 - 10000;
          else
            *v28 = 0;
          sub_17BC8(v24, (_DWORD *)(*(_DWORD *)(v26 + 24) + 32));
          v23 -= 10000;
        }
        ++v22;
      }
      while ( *((unsigned __int16 *)&v42 + v22) != 0xFFFF );
    }
    return sub_4A3D8(&v36);
  }
  return result;
}

