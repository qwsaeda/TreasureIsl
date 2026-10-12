/* ea:   0x00064D94 */
/* name: sub_64D94 */
/* size: 44 */
BOOL __fastcall sub_64D94(int a1, unsigned __int8 *a2, unsigned int a3)
{
  if ( a2 == nullptr )
    __debugbreak();
  return a3 >= 0x80 && *a2 == 10;
}

