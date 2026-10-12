/* ea:   0x0005BC14 */
/* name: sub_5BC14 */
/* size: 420 */
int __fastcall sub_5BC14(_DWORD *a1, int a2, int a3)
{
  int v5; // r0
  int v6; // r1
  bool v7; // zf
  int result; // r0
  int v9; // r0
  int v10; // r9
  int v11; // r4
  int v12; // r10
  int v13; // r3
  unsigned __int16 *v14; // r8
  int v15; // r3
  unsigned __int16 *v16; // r6
  int v17; // r2
  unsigned __int16 v18; // r1
  char v19; // r0
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
  v5 = a1[1];
  v6 = a1[2];
  v21[0] = 0;
  v21[1] = v5;
  v21[2] = 0;
  v21[3] = v6;
  sub_5E9E0(v22, v21, a2);
  v7 = v23 == v24;
  if ( v23 == v24 )
    v7 = v26 == v25;
  LOBYTE(result) = v7;
  result = (unsigned __int8)result;
  if ( (_BYTE)result == 0 )
  {
    v9 = sub_5E970(a1, v25);
    v10 = v9 + 2 * v23;
    v11 = v24 - v23;
    v12 = v26 - v25;
    if ( a3 == 0 )
      __debugbreak();
    v13 = sub_5E970(a3, v22[2]);
    result = v22[0];
    v14 = (unsigned __int16 *)(v13 + 2 * v22[0]);
    if ( v12 != 0 )
    {
      v15 = v11;
      do
      {
        v16 = v14;
        if ( v14 != &v14[v15] )
        {
          v17 = (unsigned __int16)word_8122C;
          do
          {
            v18 = *v16;
            if ( v17 == *v16 )
            {
              v19 = 0;
              v18 = *v16;
            }
            else
            {
              v19 = 1;
            }
            if ( v19 != 0 )
              *(unsigned __int16 *)((char *)v16 + v10 - (_DWORD)v14) = v18;
            ++v16;
          }
          while ( v16 != &v14[v15] );
        }
        v10 += 2 * a1[1];
        if ( a3 == 0 )
          __debugbreak();
        result = *(_DWORD *)(a3 + 4);
        --v12;
        v14 += result;
      }
      while ( v12 != 0 );
    }
  }
  return result;
}

