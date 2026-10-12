/* ea:   0x0001D114 */
/* name: sub_1D114 */
/* size: 1232 */
int __fastcall sub_1D114(_DWORD *a1)
{
  _DWORD *v1; // lr
  int v2; // r0
  int v3; // r2
  int v4; // r0
  int v5; // r7
  int v6; // r0
  int v7; // r6
  int v8; // r5
  int v9; // r4
  int v10; // r0
  int i; // r6
  int j; // r5
  int v13; // r3
  int v14; // r0
  int v15; // r0
  int v16; // r1
  unsigned __int8 v17; // r9
  int v18; // r1
  int v19; // r0
  int (*v20)(); // r1
  int v21; // r0
  int k; // r8
  int m; // r7
  int v24; // r3
  _DWORD *v25; // r3
  int v26; // r0
  int v27; // r6
  int v28; // r1
  int v29; // r4
  int v30; // r0
  int *v31; // r5
  int v32; // r0
  _DWORD *v33; // r2
  int v34; // r1
  int (*v35)(); // r0
  int v36; // r4
  _DWORD *v37; // lr
  int *v38; // r5
  int result; // r0
  int v40; // [sp+0h] [bp-78h]
  int v41; // [sp+0h] [bp-78h]
  int v43; // [sp+14h] [bp-64h] BYREF
  int v44; // [sp+18h] [bp-60h]
  int v45; // [sp+1Ch] [bp-5Ch]
  int v46; // [sp+20h] [bp-58h]
  _DWORD v47[4]; // [sp+24h] [bp-54h] BYREF
  int v48; // [sp+34h] [bp-44h] BYREF
  int v49; // [sp+38h] [bp-40h]
  int v50; // [sp+3Ch] [bp-3Ch]
  int v51; // [sp+40h] [bp-38h]
  _DWORD v52[13]; // [sp+44h] [bp-34h] BYREF

  v1 = a1;
  v2 = a1[12];
  v47[1] = 32;
  v47[0] = 0;
  v47[2] = 0;
  v47[3] = 32;
  v3 = **(_DWORD **)(v2 + 32);
  if ( v3 == 0 )
    __debugbreak();
  v4 = *(_DWORD *)(v3 + 12);
  if ( v4 == 16 )
  {
    v5 = 0;
    v1[3] = sub_5BC14;
  }
  else if ( v4 == 8 )
  {
    v5 = 0;
    v1[3] = sub_5D9C4;
  }
  else
  {
    v1[3] = 0;
    v5 = 0;
  }
  do
  {
    v6 = 0;
    v7 = v5 + 32;
    do
    {
      v50 = v6;
      v8 = v6 + 32;
      v9 = v1[2];
      v40 = v1[3];
      v10 = v1[12];
      v48 = v5;
      v49 = v5 + 32;
      v51 = v8;
      (*(void (__fastcall **)(int, int *, _DWORD, _DWORD *, int, int *, int))(*(_DWORD *)v9 + 20))(
        v9,
        &v48,
        **(_DWORD **)(v10 + 32),
        v47,
        v40,
        &dword_81230,
        1);
      v6 = v8;
      v1 = a1;
    }
    while ( v8 < 220 );
    v5 += 32;
  }
  while ( v7 < 176 );
  for ( i = *(_DWORD *)(a1[12] + 696) - 1; i >= 0; --i )
  {
    for ( j = 0; ; ++j )
    {
      v13 = v1[12];
      if ( j >= *(_DWORD *)(v13 + 692) )
        break;
      v14 = sub_53808(v13 + 52, i, j);
      (*(void (__fastcall **)(int, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)v14 + 4))(
        v14,
        a1[2],
        a1[3],
        a1[14],
        a1[15]);
      v1 = a1;
    }
  }
  v15 = v1[18];
  v43 = 0;
  v44 = 32;
  v45 = 0;
  v46 = 32;
  if ( v15 == 4 )
  {
    v16 = *(_DWORD *)(v1[13] + 36);
    if ( v16 != 0 )
    {
      v17 = *(_BYTE *)(v16 + 48);
      v18 = *(_DWORD *)(v1[8] + 4);
      if ( v18 == 0 )
        __debugbreak();
      v19 = *(_DWORD *)(v18 + 12);
      if ( v19 == 16 )
      {
        v20 = sub_5BC14;
      }
      else if ( v19 == 8 )
      {
        v20 = sub_5D9C4;
      }
      else
      {
        v20 = nullptr;
      }
      v21 = v1[12];
      v1[3] = v20;
      for ( k = *(_DWORD *)(v21 + 696) - 1; k >= 0; --k )
      {
        for ( m = 0; ; ++m )
        {
          v24 = v1[12];
          if ( m >= *(_DWORD *)(v24 + 692) )
            break;
          v25 = (_DWORD *)sub_53808(v24 + 52, k, m);
          v1 = a1;
          v26 = v25[9];
          if ( v26 != 0 && *(unsigned __int8 *)(v26 + 48) == v17 && v25 != (_DWORD *)a1[13] )
          {
            v27 = v25[4];
            v28 = a1[15];
            v48 = v25[3] + a1[14];
            v49 = v48 + 64;
            v29 = a1[2];
            v41 = a1[3];
            v30 = a1[8];
            v50 = v28 + v27;
            v51 = v28 + v27 + 32;
            (*(void (__fastcall **)(int, int *, _DWORD, int *, int, int *, int))(*(_DWORD *)v29 + 20))(
              v29,
              &v48,
              *(_DWORD *)(v30 + 4),
              &v43,
              v41,
              &dword_81230,
              1);
            v1 = a1;
          }
        }
      }
    }
  }
  v31 = **(int ***)(v1[12] + 68);
  while ( v31 != *(int **)(v1[12] + 68) )
  {
    (*(void (__fastcall **)(int, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)v31[2] + 4))(
      v31[2],
      v1[2],
      v1[3],
      v1[14],
      v1[15]);
    v31 = (int *)*v31;
    v1 = a1;
  }
  v32 = v1[22];
  if ( v32 >= 8 )
  {
    v1[22] = 0;
  }
  else
  {
    if ( v32 > 4 )
    {
      v43 = 0;
      v44 = 32;
      v45 = 32;
      v46 = 64;
    }
    v1[22] = v32 + 1;
  }
  v33 = (_DWORD *)v1[8];
  if ( *v33 == 0 )
    __debugbreak();
  v34 = *(_DWORD *)(*v33 + 12);
  if ( v34 == 16 )
  {
    v35 = sub_5BC14;
  }
  else if ( v34 == 8 )
  {
    v35 = sub_5D9C4;
  }
  else
  {
    v35 = nullptr;
  }
  v1[3] = v35;
  v52[2] = 94;
  v36 = v1[2];
  v52[0] = 56;
  v52[1] = 120;
  v52[3] = 126;
  (*(void (__fastcall **)(int, _DWORD *, _DWORD, int *, int (*)(), int *, int))(*(_DWORD *)v36 + 20))(
    v36,
    v52,
    *v33,
    &v43,
    v35,
    &dword_81230,
    1);
  v37 = a1;
  v38 = **(int ***)(a1[12] + 64);
  while ( v38 != *(int **)(v37[12] + 64) )
  {
    (*(void (__fastcall **)(int, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)v38[2] + 4))(
      v38[2],
      v37[2],
      v37[3],
      v37[14],
      v37[15]);
    v38 = (int *)*v38;
    v37 = a1;
  }
  result = v37[18];
  if ( result == 0 )
    return sub_1DB10(v37);
  return result;
}

