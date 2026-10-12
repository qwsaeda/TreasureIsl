/* ea:   0x0001A440 */
/* name: sub_1A440 */
/* size: 188 */
int __fastcall sub_1A440(_DWORD *a1)
{
  int result; // r0
  int v3; // r5
  int v4; // r0
  int v5; // r1
  int (*v6)(); // r1
  int v7; // r4
  _DWORD v8[8]; // [sp+Ch] [bp-20h] BYREF

  result = a1[261];
  v3 = result;
  if ( result != 0 )
  {
    v4 = *(_DWORD *)(result + 8);
    if ( v4 == 0 )
      __debugbreak();
    v5 = *(_DWORD *)(v4 + 12);
    if ( v5 == 16 )
    {
      v6 = sub_5BC14;
    }
    else if ( v5 == 8 )
    {
      v6 = sub_5D9C4;
    }
    else
    {
      v6 = nullptr;
    }
    a1[122] = v6;
    v8[0] = 61;
    v8[2] = 27;
    v7 = a1[1];
    v8[1] = 111;
    v8[3] = 67;
    return (*(int (__fastcall **)(int, _DWORD *, _DWORD, int, int (*)(), int *, int))(*(_DWORD *)v7 + 20))(
             v7,
             v8,
             *(_DWORD *)(v3 + 8),
             v3 + 12,
             v6,
             &dword_81230,
             1);
  }
  return result;
}

