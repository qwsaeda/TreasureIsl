/* ea:   0x000445F4 */
/* name: sub_445F4 */
/* size: 84 */
_DWORD *__fastcall sub_445F4(int a1, int a2)
{
  _DWORD **v3; // r2
  _DWORD *v4; // r3
  _DWORD *v5; // r5
  _DWORD *v6; // r1

  v3 = *(_DWORD ***)(a1 + 976);
  v4 = *v3;
  v5 = nullptr;
  while ( v4 != v3 )
  {
    v6 = v4 + 2;
    v4 = (_DWORD *)*v4;
    if ( v6[6] == a2 )
      v5 = v6;
  }
  return v5;
}

