/* ea:   0x00056E08 */
/* name: sub_56E08 */
/* size: 220 */
unsigned int __fastcall sub_56E08(int a1)
{
  unsigned int result; // r0
  unsigned __int16 *v3; // r0
  unsigned __int16 v4; // r0
  unsigned __int16 v5; // r1
  unsigned __int16 v6; // r4
  _DWORD **v7; // r0
  unsigned __int16 v8; // r0

  result = *(_DWORD *)(a1 + 12);
  if ( result != 0 )
  {
    result = *(_DWORD *)(result + 36);
    if ( result != 0 )
    {
      v3 = (unsigned __int16 *)(a1 + 136);
      if ( *(_DWORD *)(a1 + 216) == 1 )
      {
        v4 = sub_16C9C(v3, 0);
        sub_513CC(
          (_DWORD *)(*(_DWORD *)(a1 + 24) + 288),
          *(unsigned __int16 *)(*(_DWORD *)(*(_DWORD *)(a1 + 12) + 36) + 20),
          v4);
        v5 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
        return sub_31258(*(_DWORD *)(*(_DWORD *)(a1 + 12) + 36), v5);
      }
      else
      {
        v6 = sub_16C9C(v3, 0);
        v7 = (_DWORD **)sub_1DD10((_DWORD *)(*(_DWORD *)(a1 + 24) + 496), (int *)(a1 + 216));
        sub_513CC(*v7, *(unsigned __int16 *)(*(_DWORD *)(*(_DWORD *)(a1 + 12) + 36) + 20), v6);
        v8 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
        return sub_31258(*(_DWORD *)(*(_DWORD *)(a1 + 12) + 36), v8);
      }
    }
  }
  return result;
}

