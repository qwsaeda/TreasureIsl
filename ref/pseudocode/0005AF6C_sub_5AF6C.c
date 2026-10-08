/* ea:   0x0005AF6C */
/* name: sub_5AF6C */
/* size: 20 */
int __fastcall sub_5AF6C(int result, unsigned int a2)
{
  if ( a2 <= 0xFF )
    *(_DWORD *)(result + 16) = a2;
  else
    *(_DWORD *)(result + 16) = 255;
  return result;
}

