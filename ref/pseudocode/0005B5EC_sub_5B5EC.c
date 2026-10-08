/* ea:   0x0005B5EC */
/* name: sub_5B5EC */
/* size: 384 */
int __fastcall sub_5B5EC(_DWORD *a1, int a2, int a3, int a4, int a5)
{
  int v6; // r1
  unsigned __int8 v7; // r4
  bool v8; // zf
  int result; // r0
  int v10; // r0
  _WORD *v11; // r10
  int v12; // r4
  int v13; // r11
  int v14; // r3
  int v15; // r9
  int v16; // r6
  _WORD *v17; // r5
  int v18; // r4
  _DWORD v21[4]; // [sp+8h] [bp-54h] BYREF
  _DWORD v22[4]; // [sp+18h] [bp-44h] BYREF
  int v23; // [sp+28h] [bp-34h]
  int v24; // [sp+2Ch] [bp-30h]
  int v25; // [sp+30h] [bp-2Ch]
  int v26; // [sp+34h] [bp-28h]

  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == 0 || *(_DWORD *)(a3 + 12) != 16 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v6 = a1[2];
  v7 = 0;
  v21[1] = a1[1];
  v21[3] = v6;
  v21[0] = 0;
  v21[2] = 0;
  sub_5E9E0(v22, v21, a2);
  v8 = v23 == v24;
  if ( v23 == v24 )
    v8 = v26 == v25;
  if ( v8 )
    v7 = 1;
  result = v7;
  if ( v7 == 0 )
  {
    v10 = sub_5E970(a1, v25);
    v11 = (_WORD *)(v10 + 2 * v23);
    v12 = v24 - v23;
    v13 = v26 - v25;
    if ( a3 == 0 )
      __debugbreak();
    v14 = sub_5E970(a3, v22[2]);
    result = v22[0];
    v15 = v14 + 2 * v22[0];
    if ( v13 != 0 )
    {
      v16 = 2 * v12;
      do
      {
        v17 = v11;
        if ( v15 != v16 + v15 )
        {
          v18 = v15 - (_DWORD)v11;
          do
          {
            *v17 = (*(int (__fastcall **)(int, _DWORD, _DWORD))(*(_DWORD *)a5 + 8))(
                     a5,
                     *(unsigned __int16 *)((char *)v17 + v18),
                     (unsigned __int16)*v17);
            ++v17;
          }
          while ( (_WORD *)((char *)v17 + v18) != (_WORD *)(v16 + v15) );
        }
        v11 += a1[1];
        if ( a3 == 0 )
          __debugbreak();
        result = *(_DWORD *)(a3 + 4);
        --v13;
        v15 += 2 * result;
      }
      while ( v13 != 0 );
    }
  }
  return result;
}

