/* ea:   0x000443FC */
/* name: sub_443FC */
/* size: 476 */
BOOL __fastcall sub_443FC(_DWORD *a1)
{
  _DWORD *v1; // r3
  _DWORD *v2; // r2
  int v3; // r4
  int v4; // r1
  int v5; // r2
  int v6; // r0
  int v7; // r11
  int v8; // r8
  int v9; // r6
  int v10; // r7
  _DWORD *v11; // r5
  int v12; // r1
  int (*v13)(); // r2
  int v14; // r4
  _DWORD **v15; // r4
  _DWORD *v16; // r5
  BOOL result; // r0
  int v19; // [sp+10h] [bp-50h]
  int v20; // [sp+14h] [bp-4Ch]
  int v21; // [sp+18h] [bp-48h]
  _DWORD v22[4]; // [sp+1Ch] [bp-44h] BYREF
  _DWORD v23[13]; // [sp+2Ch] [bp-34h] BYREF

  v1 = a1;
  v2 = (_DWORD *)a1[247];
  if ( v2 != nullptr )
  {
    v3 = *v2;
    if ( *v2 == 0 )
      __debugbreak();
    v4 = *(_DWORD *)(*v2 + 4) / v2[1];
    v21 = v4;
    if ( v3 == 0 )
      __debugbreak();
    v5 = 0;
    v1 = a1;
    v19 = *(_DWORD *)(v3 + 8);
    v22[3] = v19;
    v6 = a1[248];
    v22[0] = 0;
    v22[1] = v4;
    v22[2] = 0;
    v20 = 0;
    if ( v6 > 0 )
    {
      v7 = 0;
      do
      {
        v8 = 0;
        if ( (int)v1[249] > 0 )
        {
          v9 = v7 + v4;
          v10 = 0;
          do
          {
            v11 = (_DWORD *)v1[247];
            if ( *v11 == 0 )
              __debugbreak();
            v12 = *(_DWORD *)(*v11 + 12);
            if ( v12 == 16 )
            {
              v13 = sub_5BC14;
            }
            else if ( v12 == 8 )
            {
              v13 = sub_5D9C4;
            }
            else
            {
              v13 = nullptr;
            }
            v14 = v1[1];
            v1[122] = v13;
            v23[3] = v10 + v19;
            v23[0] = v7;
            v23[1] = v9;
            v23[2] = v10;
            (*(void (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v14 + 20))(
              v14,
              v23,
              *v11,
              v22,
              v13,
              &dword_81230,
              1);
            ++v8;
            v1 = a1;
            v10 += v19;
          }
          while ( v8 < a1[249] );
          v4 = v21;
          v5 = v20;
        }
        v20 = ++v5;
        v7 += v4;
      }
      while ( v5 < v1[248] );
    }
  }
  v15 = (_DWORD **)v1[244];
  v16 = *v15;
  while ( 1 )
  {
    result = v16 != v15;
    if ( v16 == v15 )
      break;
    sub_460F8(v16 + 2, v1[1], v1[122]);
    v16 = (_DWORD *)*v16;
    v1 = a1;
  }
  return result;
}

