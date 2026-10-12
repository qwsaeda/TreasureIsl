/* ea:   0x00017C7C */
/* name: sub_17C7C */
/* size: 48 */
int __fastcall sub_17C7C(int a1, int a2)
{
  int result; // r0

  result = *(unsigned __int8 *)(a1 + 33);
  if ( (_BYTE)result != 0 )
    return sub_17CAC(*(_DWORD *)(a1 + 28), *(_DWORD *)(*(_DWORD *)(a1 + 48) + 12 * a2));
  return result;
}

