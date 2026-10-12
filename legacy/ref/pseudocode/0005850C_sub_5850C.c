/* ea:   0x0005850C */
/* name: sub_5850C */
/* size: 32 */
int __fastcall sub_5850C(int a1)
{
  int result; // r0

  result = *(unsigned __int8 *)(a1 + 32);
  if ( (_BYTE)result != 0 )
    return sub_59B9C(*(_DWORD *)(*(_DWORD *)(a1 + 28) + 20));
  return result;
}

